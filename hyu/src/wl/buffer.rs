use crate::{wl, Result};

pub struct Buffer {}

impl Buffer {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Buffer {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}
