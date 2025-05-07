use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct ZwlrLayerShellV1 {
	object_id: wl::Id<Self>,
	conn: Rc<Connection>,
}

impl ZwlrLayerShellV1 {
	pub fn new(object_id: wl::Id<Self>, conn: Rc<Connection>) -> Self {
		Self { object_id, conn }
	}
}

impl wl::Object for ZwlrLayerShellV1 {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_shell_v1:request:get_layer_surface
				let (id, surface, _output, _layer, _namespace): (
					wl::Id<wl::ZwlrLayerSurfaceV1>,
					wl::Id<wl::Surface>,
					wl::Id<wl::Output>,
					u32,
					String,
				) = wlm::decode::from_slice(params)?;

				client.new_object(id, wl::ZwlrLayerSurfaceV1::new(id, self.conn.clone()));

				let wl_surface = client.get_object_mut(surface)?;
				wl_surface.set_role(wl::SurfaceRole::LayerSurface {
					wlr_layer_surface: id,
					initial_commit: true,
				})?;
			}
			1 => {
				// https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_shell_v1:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ZwlrLayerShellV1"),
		}

		Ok(())
	}
}

impl wl::Global for ZwlrLayerShellV1 {
	fn get_name(&self) -> &'static str {
		"zwlr_layer_shell_v1"
	}

	fn get_version(&self) -> u32 {
		3
	}

	fn bind(&self, client: &mut Client, object_id: u32, _version: u32) -> Result<()> {
		let id = wl::Id::<Self>::new(object_id);
		client.new_object(id, Self::new(id, self.conn.clone()));

		Ok(())
	}
}
