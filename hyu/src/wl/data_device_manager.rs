use crate::{wl, Result};

#[derive(Debug)]
pub struct DataDeviceManager {}

impl DataDeviceManager {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for DataDeviceManager {
	fn handle(&self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			1 => {
				let (id, seat): (u32, u32) = wlm::decode::from_slice(&params)?;
				client.push_client_object(id, std::rc::Rc::new(wl::DataDevice::new(seat)));
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

	fn bind(&self, client: &mut wl::Client, object_id: u32) {
		client.push_client_object(object_id, std::rc::Rc::new(Self::new()));
	}
}
