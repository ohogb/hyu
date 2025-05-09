use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct Registry {
	object_id: wl::Id<Self>,
	conn: Rc<Connection>,
	display: wl::Id<wl::Display>,
}

impl Registry {
	pub fn new(
		object_id: wl::Id<Self>,
		conn: Rc<Connection>,
		display: wl::Id<wl::Display>,
	) -> Self {
		Self {
			object_id,
			conn,
			display,
		}
	}

	pub fn global(&self, name: u32, interface: impl AsRef<str>, version: u32) -> Result<()> {
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (name, interface.as_ref(), version),
		})
	}
}

impl wl::Object for Registry {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_registry:request:bind
				let (name, interface, version, client_object): (u32, String, u32, u32) =
					wlm::decode::from_slice(params)?;

				println!(" {client_object}, {name}, {interface:?} {version}");

				let display = client.get_object(self.display)?;

				let global = display.get_global(name).unwrap();
				global.bind(client, client_object, version)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Registry"),
		}

		Ok(())
	}
}
