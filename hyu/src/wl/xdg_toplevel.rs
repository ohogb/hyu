use crate::{wl, Result};

pub struct XdgToplevel {}

impl XdgToplevel {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for XdgToplevel {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}