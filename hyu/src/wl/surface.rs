use crate::{wl, Result};

pub struct Surface {}

impl Surface {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Surface {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			4 => {
				// wl_surface.set_opaque_region()
				// https://gitlab.freedesktop.org/wayland/wayland/blob/master/protocol/wayland.xml#L1518
			}
			5 => {
				// wl_surface.set_input_region()
				// https://gitlab.freedesktop.org/wayland/wayland/blob/master/protocol/wayland.xml#L1549
			}
			6 => {
				// wl_surface.commit()
				// https://gitlab.freedesktop.org/wayland/wayland/blob/master/protocol/wayland.xml#L1578
			}
			8 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_buffer_scale
				let _scale: u32 = wlm::decode::from_slice(&params)?;
			}
			_ => Err(format!("unknown op '{op}' in Surface"))?,
		}

		Ok(())
	}
}
