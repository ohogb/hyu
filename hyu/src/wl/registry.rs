use std::io::Write;

use crate::{wl, Result};

#[derive(Debug, Clone)]
pub struct Registry {
	object_id: u32,
	display: *const wl::Display,
}

impl Registry {
	pub fn new(object_id: u32, display: &wl::Display) -> Self {
		Self {
			object_id,
			display: display as _,
		}
	}

	pub fn global(&self, name: u32, interface: impl AsRef<str>, version: u32) -> Result<Vec<u8>> {
		let mut buf = Vec::new();

		buf.write_all(&self.object_id.to_ne_bytes())?;
		buf.write_all(&0u16.to_ne_bytes())?;

		let args = wlm::encode::to_vec(&(name, interface.as_ref(), version))?;

		buf.write_all(&(8u16 + args.len() as u16).to_ne_bytes())?;
		buf.extend(args);

		Ok(buf)
	}
}

impl wl::Object for Registry {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				let (name, interface, _version, client_object): (u32, String, u32, u32) =
					wlm::decode::from_slice(&params)?;

				println!(" {client_object}, {name}, {interface:?} {_version}");

				// hmm
				let global = unsafe { &*self.display }.get_global(name).unwrap();
				global.bind(client, client_object);
			}
			_ => Err(format!("unknown op '{op}' in Registry"))?,
		}

		Ok(())
	}
}
