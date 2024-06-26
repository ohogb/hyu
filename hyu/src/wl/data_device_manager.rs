use crate::{wl, Result};

#[derive(Debug)]
pub struct DataDeviceManager {}

impl DataDeviceManager {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for DataDeviceManager {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			1 => {
				// https://wayland.app/protocols/wayland#wl_data_device_manager:request:get_data_device
				let (id, seat): (wl::Id<wl::DataDevice>, wl::Id<wl::Seat>) =
					wlm::decode::from_slice(params)?;

				client.new_object(id, wl::DataDevice::new(seat));
			}
			_ => Err(format!("unknown op '{op}' in DataDeviceManager"))?,
		}

		Ok(())
	}
}

impl wl::Global for DataDeviceManager {
	fn get_name(&self) -> &'static str {
		"wl_data_device_manager"
	}

	fn get_version(&self) -> u32 {
		3
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		client.new_object(wl::Id::new(object_id), Self::new());
		Ok(())
	}
}
