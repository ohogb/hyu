#![feature(fs_try_exists, unix_socket_peek)]

mod state;
pub mod wl;

pub use state::*;
use wl::Object;

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

	let clients = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::<
		std::os::fd::RawFd,
		wl::Client,
	>::new()));

	let ptr = clients.as_ref() as *const _ as u64;

	for (index, stream) in socket.incoming().enumerate() {
		let mut stream = stream?;

		std::thread::spawn(move || {
			|| -> Result<()> {
				let ptr = ptr as *const std::sync::Mutex<
					std::collections::HashMap<std::os::fd::RawFd, wl::Client>,
				>;

				let mut client = wl::Client::new(State {
					buffer: Buffer(Vec::new()),
					start_position: (100 * (index + 1) as i32, 100 * (index + 1) as i32),
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

				unsafe {
					let mut lock = (*ptr).lock().unwrap();
					lock.insert(stream.as_raw_fd(), client);
				}

				loop {
					let fd = stream.as_raw_fd();

					let mut cmsg = nix::cmsg_space!([std::os::fd::RawFd; 10]);

					let msgs = nix::sys::socket::recvmsg::<()>(
						fd,
						&mut [],
						Some(&mut cmsg),
						nix::sys::socket::MsgFlags::empty(),
					)?;

					let mut lock = unsafe { (*ptr).lock().unwrap() };
					let client = lock.get_mut(&stream.as_raw_fd()).unwrap();

					for i in msgs.cmsgs() {
						match i {
							nix::sys::socket::ControlMessageOwned::ScmRights(x) => {
								client.push_fds(x)
							}
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

					// TODO: think how to do this the safe way
					let object = object as *mut wl::Resource;
					unsafe { (*object).handle(client, op, params)? };

					stream.write_all(&client.get_state().buffer.0)?;
					client.get_state().buffer.0.clear();

					let mut image = bmp::Image::new(2560, 1440);

					for client in lock.values_mut() {
						for window in client.get_windows() {
							let wl::Resource::XdgToplevel(window) = window else {
								panic!();
							};

							let Some(wl::Resource::XdgSurface(xdg_surface)) =
								client.get_object(window.surface)
							else {
								panic!();
							};

							let pos = xdg_surface.position;

							let Some(wl::Resource::Surface(surface)) =
								client.get_object(xdg_surface.get_surface())
							else {
								panic!();
							};

							for (x, y, width, height, bytes_per_pixel, pixels) in
								surface.get_front_buffers(client)
							{
								for (index, pixel) in
									pixels.chunks(bytes_per_pixel as _).enumerate()
								{
									let index = index as i32;
									let position = window.position;

									let x = (index % width) + position.0 - pos.0 + x;
									let y = (index / width) + position.1 - pos.1 + y;

									image.set_pixel(
										x as _,
										y as _,
										bmp::Pixel::new(pixel[2], pixel[1], pixel[0]),
									);
								}
							}
						}
					}

					image.save("image.bmp").unwrap();
				}
			}()
			.unwrap();
		});
	}

	drop(socket);
	std::fs::remove_file(path)?;

	Ok(())
}
