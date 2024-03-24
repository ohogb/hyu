use crate::{wl, Result};

pub struct XdgSurface {
	object_id: wl::Id<Self>,
	pub surface: wl::Id<wl::Surface>,
	pub position: (i32, i32),
	pub size: (i32, i32),
}

impl XdgSurface {
	pub fn new(object_id: wl::Id<Self>, surface: wl::Id<wl::Surface>) -> Self {
		Self {
			object_id,
			surface,
			position: (0, 0),
			size: (0, 0),
		}
	}

	pub fn configure(&self, client: &mut wl::Client) -> Result<()> {
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: 123u32,
		})?;

		Ok(())
	}
}

impl wl::Object for XdgSurface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_surface:request:destroy
			}
			1 => {
				// https://wayland.app/protocols/xdg-shell#xdg_surface:request:get_toplevel
				let id: wl::Id<wl::XdgToplevel> = wlm::decode::from_slice(&params)?;
				let start_position = client.start_position;

				let xdg_toplevel = wl::XdgToplevel::new(client, id, self.object_id, start_position);
				xdg_toplevel.configure(client)?;

				let surface = client.get_object_mut(self.surface)?;
				surface.set_role(wl::SurfaceRole::XdgToplevel)?;

				client.queue_new_object(id, xdg_toplevel);
			}
			3 => {
				// https://wayland.app/protocols/xdg-shell#xdg_surface:request:set_window_geometry
				let (x, y, width, height): (i32, i32, i32, i32) = wlm::decode::from_slice(&params)?;

				// TODO: double buffer
				self.position = (x, y);
				self.size = (width, height);
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
