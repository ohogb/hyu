use crate::{wl, Result};

pub struct ZwpLinuxDmabufFeedbackV1 {
	object_id: wl::Id<Self>,
}

impl ZwpLinuxDmabufFeedbackV1 {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}
}

impl wl::Object for ZwpLinuxDmabufFeedbackV1 {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			_ => Err(format!("unknown op '{op}' in ZwpLinuxDmabufFeedbackV1"))?,
		}

		Ok(())
	}
}
