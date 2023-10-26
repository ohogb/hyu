use crate::{wl, Result};

#[derive(Debug)]
pub struct Compositor {}

impl Compositor {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Compositor {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				let id: u32 = wlm::decode::from_slice(&params)?;
				client.push_client_object(id, wl::Surface::new());
			}
			1 => {
				let id: u32 = wlm::decode::from_slice(&params)?;
				client.push_client_object(id, wl::Region::new());
			}
			_ => Err(format!("unknown op '{op}' in Compositor"))?,
		}

		Ok(())
	}
}

impl wl::Global for Compositor {
	fn get_name(&self) -> &'static str {
		"wl_compositor"
	}

	fn get_version(&self) -> u32 {
		4
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) {
		client.push_client_object(object_id, Self::new());
	}
}
