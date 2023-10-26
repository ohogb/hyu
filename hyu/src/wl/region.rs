use crate::{wl, Result};

pub struct Region {
	areas: Vec<(u32, u32, u32, u32)>,
}

impl Region {
	pub fn new() -> Self {
		Self { areas: Vec::new() }
	}
}

impl wl::Object for Region {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// wl_region.destroy()
				// https://gitlab.freedesktop.org/wayland/wayland/blob/master/protocol/wayland.xml#L2637
			}
			1 => {
				let (x, y, w, h): (u32, u32, u32, u32) = wlm::decode::from_slice(&params)?;
				self.areas.push((x, y, w, h));
			}
			_ => Err(format!("unknown op '{op}' in Region"))?,
		}

		Ok(())
	}
}
