use crate::wl;

pub struct Keyboard {}

impl Keyboard {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Keyboard {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: Vec<u8>) -> crate::Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_keyboard:request:release
			}
			_ => Err(format!("unknown op '{op}' in Keyboard"))?,
		}

		Ok(())
	}
}
