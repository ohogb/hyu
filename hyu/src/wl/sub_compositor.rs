use crate::{wl, Result};

#[derive(Debug)]
pub struct SubCompositor {}

impl SubCompositor {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for SubCompositor {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_subcompositor:request:destroy
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_subcompositor:request:get_subsurface
				let (id, surface_id, parent_id): (
					wl::Id<wl::SubSurface>,
					wl::Id<wl::Surface>,
					wl::Id<wl::Surface>,
				) = wlm::decode::from_slice(params)?;

				let parent = client.get_object_mut(parent_id)?;
				parent.push(id);

				let surface = client.get_object_mut(surface_id)?;
				surface.set_role(wl::SurfaceRole::SubSurface {
					mode: wl::SubSurfaceMode::Sync,
					parent: parent_id,
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
		client.queue_new_object(wl::Id::new(object_id), Self::new());
		Ok(())
	}
}
