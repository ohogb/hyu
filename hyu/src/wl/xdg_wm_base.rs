use crate::{wl, Result};

#[derive(Debug)]
pub struct XdgWmBase {}

impl XdgWmBase {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for XdgWmBase {
	fn handle(&self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}

impl wl::Global for XdgWmBase {
	fn get_name(&self) -> &'static str {
		"xdg_wm_base"
	}

	fn get_version(&self) -> u32 {
		6
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) {
		client.push_client_object(object_id, std::rc::Rc::new(Self::new()));
	}
}
