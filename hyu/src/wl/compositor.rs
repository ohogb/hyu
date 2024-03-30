use crate::{wl, Result};

#[derive(Debug)]
pub struct Compositor {}

impl Compositor {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Compositor {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_compositor:request:create_surface
				let id: wl::Id<wl::Surface> = wlm::decode::from_slice(params)?;
				client.queue_new_object(id, wl::Surface::new(id));
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_compositor:request:create_region
				let id: wl::Id<wl::Region> = wlm::decode::from_slice(params)?;
				client.queue_new_object(id, wl::Region::new(id));
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

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		client.queue_new_object(wl::Id::new(object_id), Self::new());
		Ok(())
	}
}
