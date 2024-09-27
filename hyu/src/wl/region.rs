use crate::{wl, Client, Point, Result};

#[derive(Clone)]
pub struct Region {
	object_id: wl::Id<Self>,
	pub areas: Vec<(Point, Point)>,
}

impl Region {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self {
			object_id,
			areas: Vec::new(),
		}
	}
}

impl wl::Object for Region {
	fn handle(&mut self, client: &mut Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_region:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_region:request:add
				let (x, y, w, h): (i32, i32, i32, i32) = wlm::decode::from_slice(params)?;
				self.areas.push((Point(x, y), Point(w, h)));
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_region:request:subtract
				let (_x, _y, _w, _h): (i32, i32, i32, i32) = wlm::decode::from_slice(params)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Region"),
		}

		Ok(())
	}
}
