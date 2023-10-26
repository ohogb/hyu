use crate::{wl, Result};

pub struct XdgSurface {}

impl XdgSurface {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for XdgSurface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}
