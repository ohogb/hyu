use crate::{wl, Client, Result};

pub struct Registry {
	object_id: wl::Id<Self>,
	display: wl::Id<wl::Display>,
}

impl Registry {
	pub fn new(object_id: wl::Id<Self>, display: wl::Id<wl::Display>) -> Self {
		Self { object_id, display }
	}

	pub fn global(
		&self,
		client: &mut Client,
		name: u32,
		interface: impl AsRef<str>,
		version: u32,
	) -> Result<()> {
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (name, interface.as_ref(), version),
		})
	}
}

impl wl::Object for Registry {
	fn handle(&mut self, client: &mut Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_registry:request:bind
				let (name, interface, _version, client_object): (u32, String, u32, u32) =
					wlm::decode::from_slice(params)?;

				println!(" {client_object}, {name}, {interface:?} {_version}");

				let display = client.get_object(self.display)?;

				let global = display.get_global(name).unwrap();
				global.bind(client, client_object)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Registry"),
		}

		Ok(())
	}
}
