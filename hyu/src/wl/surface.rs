use crate::{wl, Result};

pub struct Surface {
	children: Vec<u32>,
	buffer: Option<(u32, u32, u32)>,
	front_buffer: Option<(i32, i32, i32, Vec<u8>)>,
	frame_callback: Option<u32>,
}

impl Surface {
	pub fn new() -> Self {
		Self {
			children: Vec::new(),
			buffer: None,
			front_buffer: None,
			frame_callback: None,
		}
	}

	pub fn push(&mut self, child: u32) {
		self.children.push(child);
	}

	pub fn get_front_buffers(
		&self,
		client: &wl::Client,
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
			let Some(wl::Resource::SubSurface(sub_surface)) = client.get_object(*i) else {
				panic!();
			};

			let Some(wl::Resource::Surface(surface)) = client.get_object(sub_surface.surface)
			else {
				panic!();
			};

			let position = sub_surface.position;

			ret.extend(
				surface
					.get_front_buffers(client)
					.into_iter()
					.map(|x| (x.0 + position.0, x.1 + position.1, x.2, x.3, x.4, x.5)),
			);
		}

		ret
	}
}

impl wl::Object for Surface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:destroy
			}
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

				assert!(self.frame_callback.is_none());
				self.frame_callback = Some(callback);
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
				if let Some((buffer_id, _x, _y)) = self.buffer {
					let Some(wl::Resource::Buffer(buffer)) = client.get_object_mut(buffer_id)
					else {
						panic!();
					};

					self.front_buffer = Some((
						buffer.width,
						buffer.height,
						buffer.stride / buffer.width,
						buffer.get_pixels(),
					));

					client.send_message(wlm::Message {
						object_id: buffer_id,
						op: 0,
						args: (),
					})?;

					if let Some(callback) = self.frame_callback {
						client.send_message(wlm::Message {
							object_id: callback,
							op: 0,
							args: 0u32,
						})?;

						client.remove_client_object(callback)?;
						self.frame_callback = None;
					}
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
