#![feature(fs_try_exists, unix_socket_peek)]

mod state;
pub mod wl;

pub use state::*;

use std::{
	io::{Read, Write},
	os::fd::AsRawFd,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
	let runtime_dir = std::env::var("XDG_RUNTIME_DIR")?;

	let index = 1;
	let path = std::path::PathBuf::from_iter([runtime_dir, format!("wayland-{index}")]);

	if std::fs::try_exists(&path)? {
		std::fs::remove_file(&path)?;
	}

	let socket = std::os::unix::net::UnixListener::bind(&path)?;

	for i in socket.incoming() {
		let mut stream = i?;

		let mut client = wl::Client::new(State {
			buffer: Buffer(Vec::new()),
		});

		let mut display = wl::Display::new();

		display.push_global(wl::Shm::new());
		display.push_global(wl::Compositor::new());
		display.push_global(wl::SubCompositor::new());
		display.push_global(wl::DataDeviceManager::new());
		display.push_global(wl::Seat::new());
		display.push_global(wl::Output::new());
		display.push_global(wl::XdgWmBase::new());

		client.push_client_object(1, display);

		loop {
			let fd = stream.as_raw_fd();

			let mut events = nix::libc::epoll_event { events: 0, u64: 0 };

			unsafe {
				nix::libc::epoll_wait(fd, &mut events as _, 1, -1);
			}

			let mut cmsg = nix::cmsg_space!([std::os::fd::RawFd; 10]);

			let msgs = nix::sys::socket::recvmsg::<()>(
				fd,
				&mut [],
				Some(&mut cmsg),
				nix::sys::socket::MsgFlags::empty(),
			)?;

			for i in msgs.cmsgs() {
				match i {
					nix::sys::socket::ControlMessageOwned::ScmRights(x) => client.push_fds(x),
					_ => panic!(),
				}
			}

			let mut obj = [0u8; 4];
			stream.read_exact(&mut obj).unwrap();

			let mut op = [0u8; 2];
			stream.read_exact(&mut op).unwrap();

			let mut size = [0u8; 2];
			stream.read_exact(&mut size).unwrap();

			let size = u16::from_ne_bytes(size) - 0x8;

			let mut params = Vec::new();
			let _ = (&mut stream)
				.take(size as _)
				.read_to_end(&mut params)
				.unwrap();

			let object = u32::from_ne_bytes(obj);
			let op = u16::from_ne_bytes(op);

			let Some(object) = client.get_object_mut(object) else {
				return Err(format!("unknown object '{object}'"))?;
			};

			let object = (&mut **object) as *mut dyn wl::Object;
			unsafe { (*object).handle(&mut client, op, params)? };

			stream.write_all(&client.get_state().buffer.0)?;
			client.get_state().buffer.0.clear();
		}
	}

	drop(socket);
	std::fs::remove_file(path)?;

	Ok(())
}
