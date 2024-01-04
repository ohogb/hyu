use crate::{wl, Result};

pub struct ShmPool {}

impl ShmPool {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for ShmPool {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}
