#![feature(fs_try_exists, unix_socket_peek)]

use std::io::{Read, Write};

#[derive(Clone)]
enum Resource {
	Display,
	Callback,
	Registry,
	Compositor,
	SubCompositor,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut resources = std::collections::HashMap::<u32, Resource>::new();
	let mut names = std::collections::HashMap::<u32, u32>::new();
	let mut current_name: u32 = 1;

	resources.insert(0xFF000000, Resource::Compositor);
	resources.insert(0xFF000001, Resource::SubCompositor);
	resources.insert(1, Resource::Display);

	let runtime_dir = std::env::var("XDG_RUNTIME_DIR")?;

	/*let index = std::fs::read_dir(&runtime_dir)?
	.filter_map(|x| {
		let name = x.ok()?.file_name().into_string().ok()?;

		if name.starts_with("wayland-") && !name.ends_with(".lock") {
			Some(())
		} else {
			None
		}
	})
	.count();*/

	let index = 1;
	let path = std::path::PathBuf::from_iter([runtime_dir, format!("wayland-{index}")]);

	if std::fs::try_exists(&path)? {
		std::fs::remove_file(&path)?;
	}

	let socket = std::os::unix::net::UnixListener::bind(&path)?;

	for i in socket.incoming() {
		let mut stream = i?;
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

			let Some(object) = resources.get(&object) else {
				return Err(format!("unknown object '{object}'"))?;
			};

			match object {
				Resource::Display => match op {
					0 => {
						let param = wlm::decode::from_slice(&params)?;
						resources.insert(param, Resource::Callback);

						let mut buf = Vec::new();

						buf.write_all(&param.to_ne_bytes())?;
						buf.write_all(&0u16.to_ne_bytes())?;
						buf.write_all(&(8u16 + 4u16).to_ne_bytes())?;
						buf.write_all(&(0u32).to_ne_bytes())?;

						stream.write_all(&buf)?;
					}
					1 => {
						let param = wlm::decode::from_slice(&params)?;
						resources.insert(param, Resource::Registry);

						let mut buf = Vec::new();

						buf.write_all(&param.to_ne_bytes())?;
						buf.write_all(&0u16.to_ne_bytes())?;

						let name = current_name;
						current_name += 1;
						names.insert(name, param);

						let args = wlm::encode::to_vec(&(name, "wl_compositor", 4))?;
						buf.write_all(&(8u16 + args.len() as u16).to_ne_bytes())?;

						buf.extend(args);

						println!("{}", buf.len());

						stream.write_all(&buf)?;
					}
					_ => return Err(format!("unknown op '{op}' on Display"))?,
				},
				Resource::Callback => todo!(),
				Resource::Registry => match op {
					0 => {
						let (name, interface, _version, client_object): (u32, String, u32, u32) =
							wlm::decode::from_slice(&params)?;

						println!(" {client_object}, {name}, {interface:?} {_version}");

						let object = names.get(&name).unwrap();
						let object = resources.get(object).unwrap();

						resources.insert(client_object, object.clone());
					}
					_ => return Err(format!("unknown op '{op}' on Registry"))?,
				},
				Resource::Compositor => todo!(),
				Resource::SubCompositor => todo!(),
			}
		}
	}

	drop(socket);
	std::fs::remove_file(path)?;

	Ok(())
}
