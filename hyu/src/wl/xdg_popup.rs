use crate::{wl, Point, Result};

pub struct XdgPopup {
	object_id: wl::Id<Self>,
	pub xdg_surface: wl::Id<wl::XdgSurface>,
	pub parent_xdg_surface: wl::Id<wl::XdgSurface>,
	pub position: Point,
	pub size: Point,
}

impl XdgPopup {
	pub fn new(
		object_id: wl::Id<Self>,
		xdg_surface: wl::Id<wl::XdgSurface>,
		parent_xdg_surface: wl::Id<wl::XdgSurface>,
	) -> Self {
		Self {
			object_id,
			xdg_surface,
			parent_xdg_surface,
			position: Point(0, 0),
			size: Point(0, 0),
		}
	}

	pub fn configure(
		&mut self,
		client: &mut wl::Client,
		position: Point,
		size: Point,
	) -> Result<()> {
		// https://wayland.app/protocols/xdg-shell#xdg_popup:event:configure
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (position.0, position.1, size.0, size.1),
		})?;

		self.position = position;
		self.size = size;

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
				let parent = client.get_object_mut(self.parent_xdg_surface)?;
				parent.popups.retain(|&x| x != self.object_id);

				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			1 => {
				// https://wayland.app/protocols/xdg-shell#xdg_popup:request:grab
				let (_seat, _serial): (wl::Id<wl::Seat>, u32) = wlm::decode::from_slice(params)?;
			}
			2 => {
				// https://wayland.app/protocols/xdg-shell#xdg_popup:request:reposition
				let (positioner, token): (wl::Id<wl::XdgPositioner>, u32) =
					wlm::decode::from_slice(params)?;

				self.repositioned(client, token)?;

				let positioner = client.get_object(positioner)?;
				let parent_xdg_surface = client.get_object(self.parent_xdg_surface)?;

				let (position, size) = positioner.finalize(parent_xdg_surface)?;
				self.configure(client, position, size)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in XdgPopup"),
		}

		Ok(())
	}
}
