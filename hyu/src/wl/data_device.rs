use crate::{wl, Result};

pub struct DataDevice {
	seat: wl::Id<wl::Seat>,
}

impl DataDevice {
	pub fn new(seat: wl::Id<wl::Seat>) -> Self {
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
