use crate::wl;

pub struct Keyboard {}

impl Keyboard {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Keyboard {
	fn handle(
		&mut self,
		_client: &mut wl::Client,
		_op: u16,
		_params: Vec<u8>,
	) -> crate::Result<()> {
		todo!()
	}
}
