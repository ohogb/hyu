#![feature(unix_socket_ancillary_data)]
#![feature(generic_arg_infer)]
#![feature(seek_stream_len)]

pub mod backend;
mod state;
pub mod wl;

use wl::Object;

use std::{io::Read, os::fd::AsRawFd};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn client_event_loop(mut stream: std::os::unix::net::UnixStream, index: usize) -> Result<()> {
	stream.set_nonblocking(true)?;

	let mut client = wl::Client::new(
		stream.as_raw_fd(),
		((100 * index + 10) as i32, (100 * index + 10) as i32),
	);

	let mut display = wl::Display::new(wl::Id::new(1));

	display.push_global(wl::Shm::new());
	display.push_global(wl::Compositor::new());
	display.push_global(wl::SubCompositor::new());
	display.push_global(wl::DataDeviceManager::new());
	display.push_global(wl::Seat::new(wl::Id::null()));
	display.push_global(wl::Output::new());
	display.push_global(wl::XdgWmBase::new(wl::Id::null()));

	client.queue_new_object(wl::Id::new(1), display);

	state::clients().insert(stream.as_raw_fd(), client);

	loop {
		{
			let mut clients = state::clients();
			let client = clients.get_mut(&stream.as_raw_fd()).unwrap();

			client.process_queue()?;

			let mut cmsg_buffer = [0u8; 0x20];
			let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

			cmsg.add_fds(&client.to_send_fds);
			client.to_send_fds.clear();

			let ret = stream
				.send_vectored_with_ancillary(&[std::io::IoSlice::new(&client.buffer)], &mut cmsg);

			if let Err(e) = ret {
				match e.kind() {
					std::io::ErrorKind::BrokenPipe => {
						clients.remove(&stream.as_raw_fd());
						state::add_change(state::Change::RemoveAll(stream.as_raw_fd()));

						// TODO: temp fix for pointer focus
						let mut lock = state::pointer_over();

						if let Some((fd, ..)) = *lock {
							if fd == stream.as_raw_fd() {
								*lock = None;
							}
						}

						return Ok(());
					}
					_ => {
						Err(e)?;
					}
				}
			}

			client.buffer.clear();
		}

		if state::process_focus_changes().unwrap() {
			continue;
		}

		let mut cmsg_buffer = [0u8; 0x20];
		let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

		let mut obj = [0u8; 4];

		let len = stream
			.recv_vectored_with_ancillary(&mut [std::io::IoSliceMut::new(&mut obj)], &mut cmsg);

		let len = match len {
			Ok(len) => len,
			Err(x) => match x.kind() {
				std::io::ErrorKind::WouldBlock => {
					std::thread::sleep(std::time::Duration::from_millis(10));
					continue;
				}
				_ => {
					return Err(x)?;
				}
			},
		};

		let mut clients = state::clients();

		if len == 0 {
			clients.remove(&stream.as_raw_fd());
			state::add_change(state::Change::RemoveAll(stream.as_raw_fd()));

			// TODO: temp fix for pointer focus
			let mut lock = state::pointer_over();

			if let Some((fd, ..)) = *lock {
				if fd == stream.as_raw_fd() {
					*lock = None;
				}
			}

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

		let Some(object) = client.get_resource_mut(object) else {
			return Err(format!("unknown object '{object}'"))?;
		};

		object.handle(client, op, params)?;
	}
}

fn main() -> Result<()> {
	std::thread::spawn(move || pollster::block_on(backend::wgpu::render()).unwrap());

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
