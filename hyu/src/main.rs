#![feature(fs_try_exists, unix_socket_peek)]

mod state;
pub mod wl;

pub use state::*;

use std::io::{Read, Write};

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

		client.push_client_object(1, std::rc::Rc::new(display));

		loop {
			stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;

			let mut obj = [0u8; 4];
			let len = stream.read(&mut obj).unwrap();

			if len != 4 {
				continue;
			}

			println!("obj {obj:#?}");

			let mut op = [0u8; 2];
			stream.read_exact(&mut op).unwrap();

			println!("op {op:#?}");

			let mut size = [0u8; 2];
			stream.read_exact(&mut size).unwrap();

			let size = u16::from_ne_bytes(size) - 0x8;

			println!("params size {:#?}", size);

			let mut params = Vec::new();
			let _ = (&mut stream)
				.take(size as _)
				.read_to_end(&mut params)
				.unwrap();

			println!("params {params:#?}");

			let object = u32::from_ne_bytes(obj);
			let op = u16::from_ne_bytes(op);

			let Some(object) = client.get_object(object).cloned() else {
				return Err(format!("unknown object '{object}'"))?;
			};

			object.handle(&mut client, op, params)?;

			stream.write_all(&client.get_state().buffer.0)?;
			client.get_state().buffer.0.clear();
		}
	}

	drop(socket);
	std::fs::remove_file(path)?;

	Ok(())
}
