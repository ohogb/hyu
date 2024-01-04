use crate::{wl, Result};

pub struct SubSurface {}

impl SubSurface {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for SubSurface {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			1 => {
				// https://wayland.app/protocols/wayland#wl_subsurface:request:set_position
				let (_x, _y): (u32, u32) = wlm::decode::from_slice(&params)?;
			}
			4 => {
				// wl_subsurface.set_sync()
				// https://gitlab.freedesktop.org/wayland/wayland/blob/master/protocol/wayland.xml#L2849
			}
			_ => Err(format!("unknown op '{op}' in SubSurface"))?,
		}

		Ok(())
	}
}
