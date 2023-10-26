use crate::{wl, Result};

#[derive(Debug)]
pub struct DataDevice {
	seat: u32,
}

impl DataDevice {
	pub fn new(seat: u32) -> Self {
		Self { seat }
	}
}

impl wl::Object for DataDevice {
	fn handle(&self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}
