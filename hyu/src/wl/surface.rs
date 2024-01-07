use crate::{wl, Result};

pub struct Surface {
	children: Vec<u32>,
	buffer: Option<(u32, u32, u32)>,
	front_buffer: Option<(i32, i32, i32, Vec<u8>)>,
}

impl Surface {
	pub fn new() -> Self {
		Self {
			children: Vec::new(),
			buffer: None,
			front_buffer: None,
		}
	}

	pub fn push(&mut self, child: u32) {
		self.children.push(child);
	}

	pub fn get_front_buffers(
		&self,
		client: &mut wl::Client,
	) -> Vec<(i32, i32, i32, i32, i32, Vec<u8>)> {
		let Some(front_buffer) = self.front_buffer.as_ref() else {
			return Vec::new();
		};

		let mut ret = Vec::new();
		ret.push((
			0,
			0,
			front_buffer.0,
			front_buffer.1,
			front_buffer.2,
			front_buffer.3.clone(),
		));

		for i in &self.children {
			unsafe {
				let sub_surface =
					client.get_object_mut(*i).unwrap().as_mut() as *mut _ as *mut wl::SubSurface;

				let surface = client
					.get_object_mut((*sub_surface).surface)
					.unwrap()
					.as_mut() as *mut _ as *mut wl::Surface;

				let position = (*sub_surface).position;

				ret.extend(
					(*surface)
						.get_front_buffers(client)
						.into_iter()
						.map(|x| (x.0 + position.0, x.1 + position.1, x.2, x.3, x.4, x.5)),
				);
			}
		}

		ret
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
