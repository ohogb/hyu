use crate::{wl, Result};

#[derive(Debug)]
pub struct XdgWmBase {
	object_id: u32,
}

impl XdgWmBase {
	pub fn new(object_id: u32) -> Self {
		Self { object_id }
	}

	pub fn ping(&self, client: &mut wl::Client, serial: u32) -> Result<()> {
		// https://wayland.app/protocols/xdg-shell#xdg_wm_base:event:ping
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 0,
			args: serial,
		})
	}
}

impl wl::Object for XdgWmBase {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_wm_base:request:destroy
			}
			2 => {
				// https://wayland.app/protocols/xdg-shell#xdg_wm_base:request:get_xdg_surface
				let (id, surface): (u32, u32) = wlm::decode::from_slice(&params)?;
				client.queue_new_object(id, wl::XdgSurface::new(id, surface));
			}
			3 => {
				// https://wayland.app/protocols/xdg-shell#xdg_wm_base:request:pong
				let _serial: u32 = wlm::decode::from_slice(&params)?;
			}
			_ => Err(format!("unknown op '{op}' in XdgWmBase"))?,
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

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		client.queue_new_object(object_id, Self::new(object_id));
		Ok(())
	}
}
