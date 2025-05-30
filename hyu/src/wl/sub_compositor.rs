use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct SubCompositor {
	object_id: wl::Id<Self>,
	conn: Rc<Connection>,
}

impl SubCompositor {
	pub fn new(object_id: wl::Id<Self>, conn: Rc<Connection>) -> Self {
		Self { object_id, conn }
	}
}

impl wl::Object for SubCompositor {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
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
					mode: wl::SubSurfaceMode::Sync {
						state_to_apply: Default::default(),
					},
					parent: parent_id,
				})?;

				client.new_object(
					id,
					wl::SubSurface::new(id, self.conn.clone(), surface_id, parent_id),
				);
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

	fn bind(&self, client: &mut Client, object_id: u32, _version: u32) -> Result<()> {
		client.new_object(
			wl::Id::new(object_id),
			Self::new(wl::Id::new(object_id), self.conn.clone()),
		);
		Ok(())
	}
}
