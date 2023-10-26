use crate::{wl, Result};

pub struct Surface {}

impl Surface {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Surface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}
