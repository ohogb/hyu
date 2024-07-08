#![feature(unix_socket_ancillary_data)]
#![feature(generic_arg_infer)]
#![feature(seek_stream_len)]
#![feature(sync_unsafe_cell)]

pub mod backend;
pub mod egl;
mod point;
mod state;
pub mod wl;

pub use point::*;

use wl::Object;

use std::{io::Read, os::fd::AsRawFd};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn client_event_loop(mut stream: std::os::unix::net::UnixStream, index: usize) -> Result<()> {
	stream.set_nonblocking(true)?;

	let mut client = wl::Client::new(
		stream.as_raw_fd(),
		Point((100 * index + 10) as i32, (100 * index + 10) as i32),
	);

	let mut display = wl::Display::new(wl::Id::new(1));

	display.push_global(wl::Shm::new(wl::Id::null()));
	display.push_global(wl::Compositor::new());
	display.push_global(wl::SubCompositor::new(wl::Id::null()));
	display.push_global(wl::DataDeviceManager::new());
	display.push_global(wl::Seat::new(wl::Id::null()));
	display.push_global(wl::Output::new(wl::Id::null()));
	display.push_global(wl::XdgWmBase::new(wl::Id::null()));
	display.push_global(wl::ZwpLinuxDmabufV1::new(wl::Id::null())?);
	display.push_global(wl::WpPresentation::new(wl::Id::null()));

	client.ensure_objects_capacity();
	client.new_object(wl::Id::new(1), display);

	state::CLIENTS
		.lock()
		.unwrap()
		.insert(stream.as_raw_fd(), client);

	let mut params = Vec::new();

	loop {
		let mut clients = state::CLIENTS.lock().unwrap();

		loop {
			let mut cmsg_buffer = [0u8; 0x40];
			let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

			let mut obj = [0u8; 4];

			let len = stream
				.recv_vectored_with_ancillary(&mut [std::io::IoSliceMut::new(&mut obj)], &mut cmsg);

			let len = match len {
				Ok(len) => len,
				Err(x) => match x.kind() {
					std::io::ErrorKind::WouldBlock | std::io::ErrorKind::ConnectionReset => {
						break;
					}
					_ => {
						return Err(x)?;
					}
				},
			};

			if len == 0 {
				state::CHANGES
					.lock()
					.unwrap()
					.push(state::Change::RemoveClient(stream.as_raw_fd()));
				state::process_focus_changes(&mut clients)?;

				return Ok(());
			}

			let client = clients.get_mut(&stream.as_raw_fd()).unwrap();

			for i in cmsg.messages() {
				let std::os::unix::net::AncillaryData::ScmRights(scm_rights) = i.unwrap() else {
					continue;
				};

				client.received_fds.extend(scm_rights.into_iter());
			}

			let mut op = [0u8; 2];
			stream.read_exact(&mut op)?;

			let mut size = [0u8; 2];
			stream.read_exact(&mut size)?;

			let size = u16::from_ne_bytes(size) - 0x8;

			params.resize(size as _, 0);
			stream.read_exact(&mut params)?;

			let object = u32::from_ne_bytes(obj);
			let op = u16::from_ne_bytes(op);

			client.ensure_objects_capacity();

			let Some(object) = client.get_resource_mut(object) else {
				return Err(format!("unknown object '{object}'"))?;
			};

			object.handle(client, op, &params)?;
			params.clear();
		}

		state::process_focus_changes(&mut clients)?;

		let client = clients.get_mut(&stream.as_raw_fd()).unwrap();

		if !client.buffer.is_empty() {
			let mut cmsg_buffer = [0u8; 0x20];
			let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

			cmsg.add_fds(&client.to_send_fds);
			client.to_send_fds.clear();

			let ret = stream
				.send_vectored_with_ancillary(&[std::io::IoSlice::new(&client.buffer)], &mut cmsg);

			if let Err(e) = ret {
				match e.kind() {
					std::io::ErrorKind::BrokenPipe => {
						state::CHANGES
							.lock()
							.unwrap()
							.push(state::Change::RemoveClient(stream.as_raw_fd()));
						state::process_focus_changes(&mut clients)?;

						return Ok(());
					}
					_ => {
						Err(e)?;
					}
				}
			}

			client.buffer.clear();
		}

		drop(clients);
		std::thread::sleep(std::time::Duration::from_millis(1));
	}
}

fn main() -> Result<()> {
	// std::thread::spawn(|| backend::winit::run(backend::gl::Setup).unwrap());
	std::thread::spawn(|| backend::drm::run().unwrap());
	std::thread::spawn(|| backend::input::run().unwrap());

	let runtime_dir = std::env::var("XDG_RUNTIME_DIR")?;

	let index = 1;
	let path = std::path::PathBuf::from_iter([runtime_dir, format!("wayland-{index}")]);

	if path.exists() {
		std::fs::remove_file(&path)?;
	}

	let socket = std::os::unix::net::UnixListener::bind(&path)?;

	for (index, stream) in socket.incoming().enumerate() {
		let stream = stream?;
		std::thread::spawn(move || client_event_loop(stream, index).unwrap());
	}

	drop(socket);
	std::fs::remove_file(path)?;

	Ok(())
}
