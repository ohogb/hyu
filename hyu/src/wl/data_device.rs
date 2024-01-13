use crate::{wl, Result};

#[derive(Debug)]
pub struct DataDevice {
	seat: u32,
}

impl DataDevice {
	pub fn new(seat: u32) -> Self {
		Self { seat }
	}
}

impl wl::Object for DataDevice {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: Vec<u8>) -> Result<()> {
		match op {
			2 => {
				// https://wayland.app/protocols/wayland#wl_data_device:request:release
			}
			_ => Err(format!("unknown op '{op}' in DataDevice"))?,
		}

		Ok(())
	}
}
