use crate::{wl, Result};

#[derive(Debug)]
pub struct Seat {}

impl Seat {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Seat {
	fn handle(&self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}

impl wl::Global for Seat {
	fn get_name(&self) -> &'static str {
		"wl_seat"
	}

	fn get_version(&self) -> u32 {
		7
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) {
		client.push_client_object(object_id, std::rc::Rc::new(Self::new()));
	}
}
