use crate::{wl, Result};

#[derive(Debug, Clone)]
pub struct Registry {
	object_id: u32,
	display: u32,
}

impl Registry {
	pub fn new(object_id: u32, display: u32) -> Self {
		Self { object_id, display }
	}

	pub fn global(&self, name: u32, interface: impl AsRef<str>, version: u32) -> Result<Vec<u8>> {
		let message = wlm::Message {
			object_id: self.object_id,
			op: 0,
			args: (name, interface.as_ref(), version),
		};

		message.to_vec()
	}
}

impl wl::Object for Registry {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				let (name, interface, _version, client_object): (u32, String, u32, u32) =
					wlm::decode::from_slice(&params)?;

				println!(" {client_object}, {name}, {interface:?} {_version}");

				let Some(wl::Resource::Display(display)) = client.get_object(self.display) else {
					panic!();
				};

				// hmm
				// TODO: this is very unsafe, if `bind()` pushes a new resource, `client` could get
				// reallocated.
				let global = unsafe { &*(display as *const wl::Display) }
					.get_global(name)
					.unwrap();

				global.bind(client, client_object)?;
			}
			_ => Err(format!("unknown op '{op}' in Registry"))?,
		}

		Ok(())
	}
}
