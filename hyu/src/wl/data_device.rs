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
	fn handle(&mut self, _client: &mut wl::Client, _op: u16, _params: Vec<u8>) -> Result<()> {
		todo!()
	}
}
