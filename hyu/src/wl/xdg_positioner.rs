use crate::{Client, Point, Result, state::HwState, wl};

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
	pub size: Option<Point>,
	pub anchor: Option<Direction>,
	pub anchor_rect: Option<(Point, Point)>,
	pub gravity: Option<Direction>,
	pub offset: Option<Point>,
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

	pub fn finalize(&self, xdg_surface: &wl::XdgSurface) -> Result<(Point, Point)> {
		let Some(anchor) = &self.anchor else {
			color_eyre::eyre::bail!("anchor not set");
		};

		let Some(anchor_rect) = &self.anchor_rect else {
			color_eyre::eyre::bail!("anchor rect not set");
		};

		let Some(size) = self.size else {
			color_eyre::eyre::bail!("size not set");
		};

		let factor = anchor.translation_factor();
		let mut pos = xdg_surface.position + anchor_rect.0 + anchor_rect.1.mul_f32(factor);

		if let Some(gravity) = &self.gravity {
			let factor = gravity.translation_factor();

			pos += size.mul_f32(factor);
			pos -= size;
		}

		if let Some(offset) = self.offset {
			pos += offset;
		}

		Ok((pos, size))
	}
}

impl wl::Object for XdgPositioner {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			1 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_size
				let (width, height): (i32, i32) = wlm::decode::from_slice(params)?;
				self.size = Some(Point(width, height));
			}
			2 => {
				// https://wayland.app/protocols/xdg-shell#xdg_positioner:request:set_anchor_rect
				let (x, y, width, height): (i32, i32, i32, i32) = wlm::decode::from_slice(params)?;
				self.anchor_rect = Some((Point(x, y), Point(width, height)));
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
				self.offset = Some(Point(offset.0, offset.1));
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
			_ => color_eyre::eyre::bail!("unknown op '{op}' in XdgPositioner"),
		}

		Ok(())
	}
}
