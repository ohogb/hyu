use crate::{wl, Result};

pub enum Anchor {
	Top,
	Bottom,
	Left,
	Right,
	TopLeft,
	BottomLeft,
	TopRight,
	BottomRight,
}

impl Anchor {
	pub fn try_from_index(index: u32) -> Option<Anchor> {
		Some(match index {
			1 => Self::Top,
			2 => Self::Bottom,
			3 => Self::Left,
			4 => Self::Right,
			5 => Self::TopLeft,
			6 => Self::BottomLeft,
			7 => Self::TopRight,
			8 => Self::BottomRight,
			_ => return None,
		})
	}
}

pub struct XdgPositioner {
	object_id: wl::Id<Self>,
	pub size: Option<(i32, i32)>,
	pub anchor: Option<Anchor>,
}

impl XdgPositioner {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self {
			object_id,
			size: None,
			anchor: None,
		}
	}
}

impl wl::Object for XdgPositioner {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:destroy
				client.remove_object(self.object_id)?;
			}
			1 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_size
				let (width, height): (i32, i32) = wlm::decode::from_slice(params)?;
				self.size = Some((width, height));
			}
			2 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_anchor_rect
				let (_x, _y, _width, _height): (i32, i32, i32, i32) =
					wlm::decode::from_slice(params)?;
			}
			3 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_anchor
				let anchor: u32 = wlm::decode::from_slice(params)?;
				self.anchor = Anchor::try_from_index(anchor);
			}
			4 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_gravity
				let _gravity: u32 = wlm::decode::from_slice(params)?;
			}
			5 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_constraint_adjustment
				let _constraint_adjustment: u32 = wlm::decode::from_slice(params)?;
			}
			6 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_offset
				let (_x, _y): (i32, i32) = wlm::decode::from_slice(params)?;
			}
			7 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_reactive
			}
			8 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_parent_size
				let (_parent_width, _parent_height): (i32, i32) = wlm::decode::from_slice(params)?;
			}
			9 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_parent_configure
				let _serial: u32 = wlm::decode::from_slice(params)?;
			}
			_ => Err(format!("unknown op '{op}' in XdgPositioner"))?,
		}

		Ok(())
	}
}
