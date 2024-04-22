use crate::{wl, Result};

pub struct XdgPopup {
	object_id: wl::Id<Self>,
	xdg_surface: wl::Id<wl::XdgSurface>,
}

impl XdgPopup {
	pub fn new(object_id: wl::Id<Self>, xdg_surface: wl::Id<wl::XdgSurface>) -> Self {
		Self {
			object_id,
			xdg_surface,
		}
	}

	pub fn configure(
		&self,
		client: &mut wl::Client,
		x: i32,
		y: i32,
		width: i32,
		height: i32,
	) -> Result<()> {
		// https://wayland.app/protocols/xdg-shell#xdg_popup:event:configure
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (x, y, width, height),
		})?;

		let xdg_surface = client.get_object_mut(self.xdg_surface)?;
		xdg_surface.configure(client)
	}

	pub fn repositioned(&self, client: &mut wl::Client, token: u32) -> Result<()> {
		// https://wayland.app/protocols/xdg-shell#xdg_popup:event:repositioned
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 2,
			args: token,
		})
	}
}

impl wl::Object for XdgPopup {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_popup:request:destroy
				client.remove_object(self.object_id)?;
			}
			2 => {
				// https://wayland.app/protocols/xdg-shell#xdg_popup:request:reposition
				let (_positioner, token): (wl::Id<wl::XdgPositioner>, u32) =
					wlm::decode::from_slice(params)?;

				self.repositioned(client, token)?;
				self.configure(client, 100, 100, 50, 50)?;
			}
			_ => Err(format!("unknown op '{op}' in XdgPopup"))?,
		}

		Ok(())
	}
}
