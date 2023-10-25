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
	let bytes = wlm::encode::to_vec(&(1u32, 2i32, "asdf"))?;
	println!("{bytes:#?}");

	let parsed: (u32, i32, String) = wlm::decode::from_slice(&bytes)?;
	println!("{parsed:#?}");

	if 1 + 1 == 2 {
		return Ok(());
	}

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
						assert!(params.len() == 4);
						let mut param = [0u8; 4];
						params.take(4).read_exact(&mut param)?;
						let param = u32::from_ne_bytes(param);
						resources.insert(param, Resource::Callback);

						let mut buf = Vec::new();

						buf.write_all(&param.to_ne_bytes())?;
						buf.write_all(&0u16.to_ne_bytes())?;
						buf.write_all(&(8u16 + 4u16).to_ne_bytes())?;
						buf.write_all(&(0u32).to_ne_bytes())?;

						stream.write_all(&buf)?;
					}
					1 => {
						assert!(params.len() == 4);
						let mut param = [0u8; 4];
						params.take(4).read_exact(&mut param)?;
						let param = u32::from_ne_bytes(param);
						resources.insert(param, Resource::Registry);

						let mut buf = Vec::new();

						buf.write_all(&param.to_ne_bytes())?;
						buf.write_all(&0u16.to_ne_bytes())?;
						let interface = "wl_compositor";
						buf.write_all(&(8u16 + 4 + 4 + 16 + 4).to_ne_bytes())?;

						let name = current_name;
						current_name += 1;
						names.insert(name, param);

						buf.write_all(&name.to_ne_bytes())?;

						buf.write_all(&(interface.len() as u32 + 1).to_ne_bytes())?;

						buf.write_all(interface.as_bytes())?;
						buf.write_all(&0u8.to_ne_bytes())?;
						buf.write_all(&0u8.to_ne_bytes())?;
						buf.write_all(&0u8.to_ne_bytes())?;

						buf.write_all(&4u32.to_ne_bytes())?;

						println!("{}", buf.len());

						stream.write_all(&buf)?;
					}
					_ => return Err(format!("unknown op '{op}' on Display"))?,
				},
				Resource::Callback => todo!(),
				Resource::Registry => match op {
					0 => {
						let mut name = [0u8; 4];
						params.take(4).read_exact(&mut name)?;
						let mut params = &params[4..];
						let name = u32::from_ne_bytes(name);

						let mut interface_len = [0u8; 4];
						params.take(4).read_exact(&mut interface_len)?;
						params = &params[4..];
						let interface_len = u32::from_ne_bytes(interface_len);

						let mut interface = Vec::new();
						interface.resize(interface_len as _, 0);

						params.take(interface_len as _).read_exact(&mut interface)?;
						params = &params[interface_len as _..];

						if interface_len % 4 != 0 {
							let amount = 3 - interface_len % 4;

							let mut asdf = Vec::new();
							asdf.resize(amount as _, 0);

							params
								.take(3 - interface_len as u64 % 4)
								.read_exact(&mut asdf)?;
							params = &params[amount as _..];
						}

						params = &params[1..];
						println!("{}", params.len());

						let mut version = [0u8; 4];
						params.take(4).read_exact(&mut version)?;
						params = &params[4..];
						let version = u32::from_ne_bytes(version);

						let mut object = [0u8; 4];
						params.take(4).read_exact(&mut object)?;
						let client_object = u32::from_ne_bytes(object);

						println!(" {client_object}, {name}, {interface:?} {version}");

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
