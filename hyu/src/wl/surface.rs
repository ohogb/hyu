use crate::{wl, Result};

pub struct Surface {}

impl Surface {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Surface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			4 => {
				// wl_surface.set_opaque_region()
				// https://gitlab.freedesktop.org/wayland/wayland/blob/master/protocol/wayland.xml#L1518
			}
			6 => {
				// wl_surface.commit()
				// https://gitlab.freedesktop.org/wayland/wayland/blob/master/protocol/wayland.xml#L1578
			}
			_ => Err(format!("unknown op '{op}' in Surface"))?,
		}

		Ok(())
	}
}
