use crate::{wl, Result};

pub struct SubSurface {
	object_id: u32,
	pub surface: u32,
	pub position: (i32, i32),
}

impl SubSurface {
	pub fn new(object_id: u32, surface: u32) -> Self {
		Self {
			object_id,
			surface,
			position: (0, 0),
		}
	}
}

impl wl::Object for SubSurface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:destroy
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:set_position
				let (x, y): (i32, i32) = wlm::decode::from_slice(&params)?;
				self.position = (x, y);
			}
			4 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:set_sync
				let surface = client.get_object_mut::<wl::Surface>(self.surface)?;

				let Some(wl::SurfaceRole::SubSurface { mode }) = &mut surface.role else {
					panic!();
				};

				*mode = wl::SubSurfaceMode::Sync;
			}
			5 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:set_desync
				let surface = client.get_object_mut::<wl::Surface>(self.surface)?;

				let Some(wl::SurfaceRole::SubSurface { mode }) = &mut surface.role else {
					panic!();
				};

				*mode = wl::SubSurfaceMode::Desync;
			}
			_ => Err(format!("unknown op '{op}' in SubSurface"))?,
		}

		Ok(())
	}
}
