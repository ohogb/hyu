use crate::{wl, Result};

pub struct SubSurface {
	object_id: wl::Id<Self>,
	pub surface: wl::Id<wl::Surface>,
	pub position: (i32, i32),
}

impl SubSurface {
	pub fn new(object_id: wl::Id<Self>, surface: wl::Id<wl::Surface>) -> Self {
		Self {
			object_id,
			surface,
			position: (0, 0),
		}
	}
}

impl wl::Object for SubSurface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:destroy
				client.queue_remove_object(self.object_id);
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:set_position
				let (x, y): (i32, i32) = wlm::decode::from_slice(params)?;
				self.position = (x, y);
			}
			4 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:set_sync
				let surface = client.get_object_mut(self.surface)?;

				let Some(wl::SurfaceRole::SubSurface { mode, .. }) = &mut surface.role else {
					panic!();
				};

				*mode = wl::SubSurfaceMode::Sync;
			}
			5 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:set_desync
				let surface = client.get_object_mut(self.surface)?;

				let Some(wl::SurfaceRole::SubSurface { mode, .. }) = &mut surface.role else {
					panic!();
				};

				*mode = wl::SubSurfaceMode::Desync;
			}
			_ => Err(format!("unknown op '{op}' in SubSurface"))?,
		}

		Ok(())
	}
}
