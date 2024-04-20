use crate::{wl, Result};

pub struct ZwpLinuxBufferParamsV1 {
	object_id: wl::Id<Self>,
}

impl ZwpLinuxBufferParamsV1 {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}
}

impl wl::Object for ZwpLinuxBufferParamsV1 {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			_ => Err(format!("unknown op '{op}' in ZwpLinuxBufferParamsV1"))?,
		}

		Ok(())
	}
}
