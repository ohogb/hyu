use crate::{wl, Result};

#[derive(Debug)]
pub struct Output {}

impl Output {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Output {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}

impl wl::Global for Output {
	fn get_name(&self) -> &'static str {
		"wl_output"
	}

	fn get_version(&self) -> u32 {
		3
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) {
		client.push_client_object(object_id, Self::new());
	}
}
