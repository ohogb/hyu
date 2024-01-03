use crate::wl;

pub struct Keyboard {}

impl Keyboard {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Keyboard {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> crate::Result<()> {
		todo!()
	}
}
