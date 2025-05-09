use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct Compositor {
	conn: Rc<Connection>,
}

impl Compositor {
	pub fn new(conn: Rc<Connection>) -> Self {
		Self { conn }
	}
}

impl wl::Object for Compositor {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_compositor:request:create_surface
				let id: wl::Id<wl::Surface> = wlm::decode::from_slice(params)?;
				client.new_object(id, wl::Surface::new(id, self.conn.clone()));
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_compositor:request:create_region
				let id: wl::Id<wl::Region> = wlm::decode::from_slice(params)?;
				client.new_object(id, wl::Region::new(id, self.conn.clone()));
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Compositor"),
		}

		Ok(())
	}
}

impl wl::Global for Compositor {
	fn get_name(&self) -> &'static str {
		"wl_compositor"
	}

	fn get_version(&self) -> u32 {
		4
	}

	fn bind(&self, client: &mut Client, object_id: u32, _version: u32) -> Result<()> {
		client.new_object(wl::Id::new(object_id), Self::new(self.conn.clone()));
		Ok(())
	}
}
