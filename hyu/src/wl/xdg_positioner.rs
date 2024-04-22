use crate::{wl, Result};

pub struct XdgPositioner {
	object_id: wl::Id<Self>,
}

impl XdgPositioner {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}
}

impl wl::Object for XdgPositioner {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			1 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_size
				let (_width, _height): (i32, i32) = wlm::decode::from_slice(params)?;
			}
			2 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_anchor_rect
				let (_x, _y, _width, _height): (i32, i32, i32, i32) =
					wlm::decode::from_slice(params)?;
			}
			3 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_anchor
				let _anchor: u32 = wlm::decode::from_slice(params)?;
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
			_ => Err(format!("unknown op '{op}' in XdgPositioner"))?,
		}

		Ok(())
	}
}
