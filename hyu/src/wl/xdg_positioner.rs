use crate::{wl, Result};

pub enum Direction {
	Top,
	Bottom,
	Left,
	Right,
	TopLeft,
	BottomLeft,
	TopRight,
	BottomRight,
}

impl Direction {
	pub fn try_from_index(index: u32) -> Option<Direction> {
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

	pub fn translation_factor(&self) -> (f32, f32) {
		match self {
			Direction::Top => (0.5, 0.0),
			Direction::Bottom => (0.5, 1.0),
			Direction::Left => (0.0, 0.5),
			Direction::Right => (1.0, 0.5),
			Direction::TopLeft => (0.0, 0.0),
			Direction::BottomLeft => (0.0, 1.0),
			Direction::TopRight => (1.0, 0.0),
			Direction::BottomRight => (1.0, 1.0),
		}
	}
}

pub struct XdgPositioner {
	object_id: wl::Id<Self>,
	pub size: Option<(i32, i32)>,
	pub anchor: Option<Direction>,
	pub anchor_rect: Option<((i32, i32), (i32, i32))>,
	pub gravity: Option<Direction>,
	pub offset: Option<(i32, i32)>,
}

impl XdgPositioner {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self {
			object_id,
			size: None,
			anchor: None,
			anchor_rect: None,
			gravity: None,
			offset: None,
		}
	}

	pub fn finalize(&self, xdg_surface: &wl::XdgSurface) -> Result<((i32, i32), (i32, i32))> {
		let Some(anchor) = &self.anchor else {
			return Err("anchor not set")?;
		};

		let Some(anchor_rect) = &self.anchor_rect else {
			return Err("anchor rect not set")?;
		};

		let Some(size) = self.size else {
			return Err("size not set")?;
		};

		let (sx, sy) = (
			xdg_surface.position.0 + anchor_rect.0 .0,
			xdg_surface.position.1 + anchor_rect.0 .1,
		);

		let (sw, sh) = (anchor_rect.1 .0, anchor_rect.1 .1);

		let factor = anchor.translation_factor();
		let mut pos = (
			sx + (sw as f32 * factor.0) as i32,
			sy + (sh as f32 * factor.1) as i32,
		);

		let offset = self.offset.unwrap_or((0, 0));

		if let Some(gravity) = &self.gravity {
			let factor = gravity.translation_factor();

			pos.0 += (size.0 as f32 * factor.0) as i32;
			pos.1 += (size.1 as f32 * factor.1) as i32;

			pos.0 -= size.0;
			pos.1 -= size.1;
		}

		pos.0 += offset.0;
		pos.1 += offset.1;

		Ok((pos, size))
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
				let (x, y, width, height): (i32, i32, i32, i32) = wlm::decode::from_slice(params)?;
				self.anchor_rect = Some(((x, y), (width, height)));
			}
			3 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_anchor
				let anchor: u32 = wlm::decode::from_slice(params)?;
				self.anchor = Direction::try_from_index(anchor);
			}
			4 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_gravity
				let gravity: u32 = wlm::decode::from_slice(params)?;
				self.gravity = Direction::try_from_index(gravity);
			}
			5 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_constraint_adjustment
				let _constraint_adjustment: u32 = wlm::decode::from_slice(params)?;
			}
			6 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_offset
				let offset: (i32, i32) = wlm::decode::from_slice(params)?;
				self.offset = Some(offset);
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
