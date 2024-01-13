use crate::wl;

pub struct Pointer {}

impl Pointer {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Pointer {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: Vec<u8>) -> crate::Result<()> {
		match op {
			1 => {
				// https://wayland.app/protocols/wayland#wl_pointer:request:release
			}
			_ => Err(format!("unknown op '{op}' in Pointer"))?,
		}

		Ok(())
	}
}
