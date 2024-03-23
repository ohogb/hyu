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
				// https://wayland.app/protocols/wayland#wl_subcompositor:request:destroy
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_subcompositor:request:get_subsurface
				let (id, surface_id, parent_id): (u32, u32, u32) =
					wlm::decode::from_slice(&params)?;

				let parent = client.get_object_mut::<wl::Surface>(parent_id)?;
				parent.push(id);

				let surface = client.get_object_mut::<wl::Surface>(surface_id)?;
				surface.set_role(wl::SurfaceRole::SubSurface {
					mode: wl::SubSurfaceMode::Sync,
				})?;

				client.queue_new_object(id, wl::SubSurface::new(id, surface_id));
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
		client.queue_new_object(object_id, Self::new());
		Ok(())
	}
}
