use crate::{wl, Result};

pub struct XdgSurface {
	object_id: u32,
}

impl XdgSurface {
	pub fn new(object_id: u32) -> Self {
		Self { object_id }
	}

	pub fn configure(&self, client: &mut wl::Client) -> Result<()> {
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 0,
			args: 123u32,
		})?;

		Ok(())
	}
}

impl wl::Object for XdgSurface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			1 => {
				let id: u32 = wlm::decode::from_slice(&params)?;

				let xdg_toplevel = wl::XdgToplevel::new(id, self);
				xdg_toplevel.configure(client)?;

				client.push_client_object(id, xdg_toplevel);
			}
			3 => {
				// https://wayland.app/protocols/xdg-shell#xdg_surface:request:set_window_geometry
				let (_x, _y, _width, _height): (u32, u32, u32, u32) =
					wlm::decode::from_slice(&params)?;
			}
			4 => {
				// https://wayland.app/protocols/xdg-shell#xdg_surface:request:ack_configure
				let _serial: u32 = wlm::decode::from_slice(&params)?;
			}
			_ => Err(format!("unknown op '{op}' in XdgSurface"))?,
		}

		Ok(())
	}
}
