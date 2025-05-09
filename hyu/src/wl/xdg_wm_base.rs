use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct XdgWmBase {
	object_id: wl::Id<Self>,
	conn: Rc<Connection>,
}

impl XdgWmBase {
	pub fn new(object_id: wl::Id<Self>, conn: Rc<Connection>) -> Self {
		Self { object_id, conn }
	}

	pub fn ping(&self, serial: u32) -> Result<()> {
		// https://wayland.app/protocols/xdg-shell#xdg_wm_base:event:ping
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: serial,
		})
	}
}

impl wl::Object for XdgWmBase {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_wm_base:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			1 => {
				// https://wayland.app/protocols/xdg-shell#xdg_wm_base:request:create_positioner
				let id: wl::Id<wl::XdgPositioner> = wlm::decode::from_slice(params)?;
				client.new_object(id, wl::XdgPositioner::new(id, self.conn.clone()));
			}
			2 => {
				// https://wayland.app/protocols/xdg-shell#xdg_wm_base:request:get_xdg_surface
				let (id, surface): (wl::Id<wl::XdgSurface>, wl::Id<wl::Surface>) =
					wlm::decode::from_slice(params)?;

				client.new_object(id, wl::XdgSurface::new(id, self.conn.clone(), surface));
			}
			3 => {
				// https://wayland.app/protocols/xdg-shell#xdg_wm_base:request:pong
				let _serial: u32 = wlm::decode::from_slice(params)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in XdgWmBase"),
		}

		Ok(())
	}
}

impl wl::Global for XdgWmBase {
	fn get_name(&self) -> &'static str {
		"xdg_wm_base"
	}

	fn get_version(&self) -> u32 {
		6
	}

	fn bind(&self, client: &mut Client, object_id: u32, _version: u32) -> Result<()> {
		client.new_object(
			wl::Id::new(object_id),
			Self::new(wl::Id::new(object_id), self.conn.clone()),
		);
		Ok(())
	}
}
