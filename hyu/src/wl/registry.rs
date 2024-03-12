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

	pub fn global(
		&self,
		client: &mut wl::Client,
		name: u32,
		interface: impl AsRef<str>,
		version: u32,
	) -> Result<()> {
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 0,
			args: (name, interface.as_ref(), version),
		})
	}
}

impl wl::Object for Registry {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_registry:request:bind
				let (name, interface, _version, client_object): (u32, String, u32, u32) =
					wlm::decode::from_slice(&params)?;

				println!(" {client_object}, {name}, {interface:?} {_version}");

				let display = client.get_object::<wl::Display>(self.display)?;

				// hmm
				// TODO: this is very unsafe, if `bind()` pushes a new resource, `client` could get
				// reallocated.
				let global = display.get_global(name).unwrap();
				global.bind(client, client_object)?;
			}
			_ => Err(format!("unknown op '{op}' in Registry"))?,
		}

		Ok(())
	}
}
