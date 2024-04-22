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
			_ => Err(format!("unknown op '{op}' in XdgPositioner"))?,
		}

		Ok(())
	}
}
