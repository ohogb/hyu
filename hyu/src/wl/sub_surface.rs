use crate::{wl, Result};

pub struct SubSurface {}

impl SubSurface {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for SubSurface {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: Vec<u8>) -> Result<()> {
		match op {
			4 => {
				// wl_subsurface.set_sync()
				// https://gitlab.freedesktop.org/wayland/wayland/blob/master/protocol/wayland.xml#L2849
			}
			_ => Err(format!("unknown op '{op}' in SubSurface"))?,
		}

		Ok(())
	}
}
