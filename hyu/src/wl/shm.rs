use crate::{wl, Result};

#[derive(Debug, Clone)]
pub struct Shm {}

impl Shm {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Shm {
	fn handle(&mut self, _client: &mut wl::Client, _op: u16, _params: Vec<u8>) -> Result<()> {
		todo!()
	}
}

impl wl::Global for Shm {
	fn get_name(&self) -> &'static str {
		"wl_shm"
	}

	fn get_version(&self) -> u32 {
		1
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		client.push_client_object(object_id, Self::new());

		client.send_message(wlm::Message {
			object_id,
			op: 0,
			args: 0u32,
		})?;

		client.send_message(wlm::Message {
			object_id,
			op: 0,
			args: 1u32,
		})?;

		Ok(())
	}
}
