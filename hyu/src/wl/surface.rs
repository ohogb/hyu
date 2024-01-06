use crate::{wl, Result};

pub struct Surface {
	buffer: Option<(u32, u32, u32)>,
	front_buffer: Option<(i32, i32, i32, Vec<u8>)>,
}

impl Surface {
	pub fn new() -> Self {
		Self {
			buffer: None,
			front_buffer: None,
		}
	}

	pub fn get_front_buffer(&self) -> Option<&(i32, i32, i32, Vec<u8>)> {
		self.front_buffer.as_ref()
	}
}

impl wl::Object for Surface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			1 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:attach
				let (buffer, x, y): (u32, u32, u32) = wlm::decode::from_slice(&params)?;

				self.buffer = if buffer != 0 {
					Some((buffer, x, y))
				} else {
					None
				};
			}
			3 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:frame
				let callback: u32 = wlm::decode::from_slice(&params)?;
				client.send_message(wlm::Message {
					object_id: callback,
					op: 0,
					args: 0u32,
				})?;
			}
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
				if let Some((buffer, _x, _y)) = &self.buffer {
					let asdf = client.get_object_mut(*buffer).unwrap();
					let asdf = unsafe { &mut *(asdf.as_mut() as *mut _ as *mut wl::Buffer) };
					self.front_buffer = Some((
						asdf.width,
						asdf.height,
						asdf.stride / asdf.width,
						asdf.get_pixels(),
					));

					/*client.send_message(wlm::Message {
						object_id: *buffer,
						op: 0,
						args: (),
					})?;*/
				}
			}
			8 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_buffer_scale
				let _scale: u32 = wlm::decode::from_slice(&params)?;
			}
			9 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:damage_buffer
				let (_x, _y, _width, _height): (u32, u32, u32, u32) =
					wlm::decode::from_slice(&params)?;
			}
			_ => Err(format!("unknown op '{op}' in Surface"))?,
		}

		Ok(())
	}
}
