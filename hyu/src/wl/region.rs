use crate::{wl, Result};

pub struct Region {}

impl Region {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Region {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}
