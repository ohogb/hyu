use crate::wl;

pub struct Pointer {}

impl Pointer {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Pointer {
	fn handle(
		&mut self,
		_client: &mut wl::Client,
		_op: u16,
		_params: Vec<u8>,
	) -> crate::Result<()> {
		todo!()
	}
}
