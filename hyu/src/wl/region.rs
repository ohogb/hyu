use crate::{wl, Result};

#[derive(Clone)]
pub struct Region {
	object_id: wl::Id<Self>,
	pub areas: Vec<(u32, u32, u32, u32)>,
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
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_region:request:destroy
				client.queue_remove_object(self.object_id);
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_region:request:add
				let (x, y, w, h): (u32, u32, u32, u32) = wlm::decode::from_slice(&params)?;
				self.areas.push((x, y, w, h));
			}
			_ => Err(format!("unknown op '{op}' in Region"))?,
		}

		Ok(())
	}
}
