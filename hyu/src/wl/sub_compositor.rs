use crate::{wl, Result};

pub struct SubCompositor {
	object_id: wl::Id<Self>,
}

impl SubCompositor {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}
}

impl wl::Object for SubCompositor {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_subcompositor:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
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

				client.new_object(id, wl::SubSurface::new(id, surface_id, parent_id));
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in SubCompositor"),
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
		client.new_object(wl::Id::new(object_id), Self::new(wl::Id::new(object_id)));
		Ok(())
	}
}
