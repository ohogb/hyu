use crate::{wl, Result};

#[derive(Debug, Clone)]
pub struct Shm {}

impl Shm {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Shm {
	fn handle(&self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
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

	fn bind(&self, client: &mut wl::Client, object_id: u32) {
		client.push_client_object(object_id, std::rc::Rc::new(Self::new()));
	}
}