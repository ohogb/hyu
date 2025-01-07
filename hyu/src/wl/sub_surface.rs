use crate::{Client, Point, Result, state::HwState, wl};

pub struct SubSurface {
	object_id: wl::Id<Self>,
	pub surface: wl::Id<wl::Surface>,
	pub parent_surface: wl::Id<wl::Surface>,
	pub position: Point,
}

impl SubSurface {
	pub fn new(
		object_id: wl::Id<Self>,
		surface: wl::Id<wl::Surface>,
		parent_surface: wl::Id<wl::Surface>,
	) -> Self {
		Self {
			object_id,
			surface,
			parent_surface,
			position: Point(0, 0),
		}
	}
}

impl wl::Object for SubSurface {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:destroy
				if let Ok(parent) = client.get_object_mut(self.parent_surface) {
					parent.children.retain(|&x| x != self.object_id);
				}

				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:set_position
				let (x, y): (i32, i32) = wlm::decode::from_slice(params)?;
				self.position = Point(x, y);
			}
			4 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:set_sync
				let surface = client.get_object_mut(self.surface)?;

				let Some(wl::SurfaceRole::SubSurface { mode, .. }) = &mut surface.role else {
					panic!();
				};

				*mode = wl::SubSurfaceMode::Sync {
					state_to_apply: Default::default(),
				};
			}
			5 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:set_desync
				let surface = client.get_object_mut(self.surface)?;

				let Some(wl::SurfaceRole::SubSurface { mode, .. }) = &mut surface.role else {
					panic!();
				};

				*mode = wl::SubSurfaceMode::Desync;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in SubSurface"),
		}

		Ok(())
	}
}
