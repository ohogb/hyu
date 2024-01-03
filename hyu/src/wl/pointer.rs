use crate::wl;

pub struct Pointer {}

impl Pointer {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Pointer {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> crate::Result<()> {
		todo!()
	}
}
