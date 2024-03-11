use crate::{wl, Result};

pub struct Surface {
	pub object_id: u32,
	pub children: Vec<u32>,
	pending_buffer: Option<u32>,
	current_buffer: Option<u32>,
	pending_frame_callback: Option<u32>,
	current_frame_callback: Option<u32>,
	pub data: Option<(i32, i32, i32, Vec<u8>)>,
}

impl Surface {
	pub fn new(object_id: u32) -> Self {
		Self {
			object_id,
			children: Vec::new(),
			pending_buffer: None,
			current_buffer: None,
			pending_frame_callback: None,
			current_frame_callback: None,
			data: None,
		}
	}

	pub fn push(&mut self, child: u32) {
		self.children.push(child);
	}

	pub fn get_front_buffers(
		&self,
		client: &wl::Client,
	) -> Vec<(i32, i32, i32, i32, i32, Vec<u8>)> {
		let Some(data) = self.data.as_ref() else {
			return Vec::new();
		};

		let mut ret = Vec::new();
		ret.push((0, 0, data.0, data.1, data.2, data.3.clone()));

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

	pub fn frame(&mut self, ms: u32, client: &mut wl::Client) -> Result<()> {
		if let Some(buffer_id) = self.current_buffer {
			let Some(wl::Resource::Buffer(buffer)) = client.get_object_mut(buffer_id) else {
				panic!();
			};

			self.data = Some((
				buffer.width,
				buffer.height,
				buffer.stride / buffer.width,
				buffer.get_pixels(),
			));

			buffer.release(client)?;
			self.current_buffer = None;
		}

		if let Some(callback) = self.current_frame_callback {
			client.send_message(wlm::Message {
				object_id: callback,
				op: 0,
				args: ms,
			})?;

			client.remove_client_object(callback)?;
			self.current_frame_callback = None;
		}

		for i in &self.children {
			let Some(wl::Resource::SubSurface(sub_surface)) = client.get_object(*i) else {
				panic!();
			};

			let Some(wl::Resource::Surface(surface)) = client.get_object_mut(sub_surface.surface)
			else {
				panic!();
			};

			surface.frame(ms, client)?;
		}

		Ok(())
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

				assert!(x == 0);
				assert!(y == 0);

				self.pending_buffer = if buffer != 0 { Some(buffer) } else { None };
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:damage
			}
			3 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:frame
				let callback: u32 = wlm::decode::from_slice(&params)?;

				assert!(self.pending_frame_callback.is_none());
				self.pending_frame_callback = Some(callback);
			}
			4 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_opaque_region
			}
			5 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_input_region
			}
			6 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:commit
				if let Some(buffer_id) = self.pending_buffer {
					self.current_buffer = Some(buffer_id);
					self.pending_buffer = None;
				}

				if let Some(frame_callback) = self.pending_frame_callback {
					self.current_frame_callback = Some(frame_callback);
					self.pending_frame_callback = None;
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
