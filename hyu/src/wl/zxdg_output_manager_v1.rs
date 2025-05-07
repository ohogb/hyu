use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct ZxdgOutputManagerV1 {
	object_id: wl::Id<Self>,
	conn: Rc<Connection>,
	version: u32,
}

impl ZxdgOutputManagerV1 {
	pub fn new(object_id: wl::Id<Self>, conn: Rc<Connection>, version: u32) -> Self {
		Self {
			object_id,
			conn,
			version,
		}
	}
}

impl wl::Object for ZxdgOutputManagerV1 {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-output-unstable-v1#zxdg_output_manager_v1:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			1 => {
				// https://wayland.app/protocols/xdg-output-unstable-v1#zxdg_output_manager_v1:request:get_xdg_output
				let (id, output): (wl::Id<wl::ZxdgOutputV1>, wl::Id<wl::Output>) =
					wlm::decode::from_slice(params)?;

				let xdg_output =
					client.new_object(id, wl::ZxdgOutputV1::new(id, self.conn.clone(), output));
				xdg_output.logical_position(0, 0)?;
				xdg_output.logical_size(2560, 1440)?;

				if self.version < 3 {
					xdg_output.done()?;
				} else {
					let wl_output = client.get_object_mut(output)?;
					wl_output.done()?;
				}
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ZxdgOutputManagerV1"),
		}

		Ok(())
	}
}

impl wl::Global for ZxdgOutputManagerV1 {
	fn get_name(&self) -> &'static str {
		"zxdg_output_manager_v1"
	}

	fn get_version(&self) -> u32 {
		3
	}

	fn bind(&self, client: &mut Client, object_id: u32, version: u32) -> Result<()> {
		let id = wl::Id::<Self>::new(object_id);
		client.new_object(id, Self::new(id, self.conn.clone(), version));

		Ok(())
	}
}
