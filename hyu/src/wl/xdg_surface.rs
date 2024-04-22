use crate::{wl, Result};

pub struct XdgSurface {
	object_id: wl::Id<Self>,
	pub surface: wl::Id<wl::Surface>,
	pub position: (i32, i32),
	pub size: (i32, i32),
	serial: u32,
}

impl XdgSurface {
	pub fn new(object_id: wl::Id<Self>, surface: wl::Id<wl::Surface>) -> Self {
		Self {
			object_id,
			surface,
			position: (0, 0),
			size: (0, 0),
			serial: 0,
		}
	}

	pub fn configure(&mut self, client: &mut wl::Client) -> Result<()> {
		// https://wayland.app/protocols/xdg-shell#xdg_surface:event:configure
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: self.serial(),
		})
	}

	fn serial(&mut self) -> u32 {
		let ret = self.serial;
		self.serial += 1;

		ret
	}
}

impl wl::Object for XdgSurface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_surface:request:destroy
			}
			1 => {
				// https://wayland.app/protocols/xdg-shell#xdg_surface:request:get_toplevel
				let id: wl::Id<wl::XdgToplevel> = wlm::decode::from_slice(params)?;

				let xdg_toplevel = client.new_object(
					id,
					wl::XdgToplevel::new(id, self.object_id, client.start_position, client.fd),
				);

				xdg_toplevel.configure_bounds(client, 1280, 720)?;
				xdg_toplevel.configure(client, 0, 0, &[])?;

				let surface = client.get_object_mut(self.surface)?;
				surface.set_role(wl::SurfaceRole::XdgToplevel)?;
			}
			2 => {
				// https://wayland.app/protocols/xdg-shell#xdg_surface:request:get_popup
				let (id, _parent, _positioner): (
					wl::Id<wl::XdgPopup>,
					wl::Id<wl::XdgSurface>,
					wl::Id<wl::XdgPositioner>,
				) = wlm::decode::from_slice(params)?;

				let xdg_popup = client.new_object(id, wl::XdgPopup::new(id, self.object_id));
				xdg_popup.configure(client, 100, 100, 50, 50)?;

				let surface = client.get_object_mut(self.surface)?;
				surface.set_role(wl::SurfaceRole::XdgPopup)?;
			}
			3 => {
				// https://wayland.app/protocols/xdg-shell#xdg_surface:request:set_window_geometry
				let (x, y, width, height): (i32, i32, i32, i32) = wlm::decode::from_slice(params)?;

				// TODO: double buffer
				self.position = (x, y);
				self.size = (width, height);
			}
			4 => {
				// https://wayland.app/protocols/xdg-shell#xdg_surface:request:ack_configure
				let _serial: u32 = wlm::decode::from_slice(params)?;
			}
			_ => Err(format!("unknown op '{op}' in XdgSurface"))?,
		}

		Ok(())
	}
}
