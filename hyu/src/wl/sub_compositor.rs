use crate::{wl, Result};

#[derive(Debug)]
pub struct SubCompositor {}

impl SubCompositor {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for SubCompositor {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:destroy
			}
			1 => {
				let (id, surface, parent): (u32, u32, u32) = wlm::decode::from_slice(&params)?;

				if let Some(wl::Resource::Surface(surface)) = client.get_object_mut(parent) {
					surface.push(id);
				} else {
					panic!();
				}

				client.push_client_object(id, wl::SubSurface::new(id, surface));
			}
			_ => Err(format!("unknown op '{op}' in SubCompositor"))?,
		}

		Ok(())
	}
}

impl wl::Global for SubCompositor {
	fn get_name(&self) -> &'static str {
		"wl_subcompositor"
	}

	fn get_version(&self) -> u32 {
		1
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		client.push_client_object(object_id, Self::new());
		Ok(())
	}
}
