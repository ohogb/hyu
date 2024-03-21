use crate::{wl, Result};

pub struct Surface {
	pub object_id: u32,
	pub children: Vec<u32>,
	pending_buffer: Option<u32>,
	current_buffer: Option<u32>,
	pending_frame_callbacks: Vec<u32>,
	current_frame_callbacks: Vec<u32>,
	pub data: Option<(i32, i32, (wgpu::Texture, wgpu::BindGroup))>,
}

impl Surface {
	pub fn new(object_id: u32) -> Self {
		Self {
			object_id,
			children: Vec::new(),
			pending_buffer: None,
			current_buffer: None,
			pending_frame_callbacks: Vec::new(),
			current_frame_callbacks: Vec::new(),
			data: None,
		}
	}

	pub fn push(&mut self, child: u32) {
		self.children.push(child);
	}

	pub fn get_front_buffers(&self, client: &wl::Client) -> Vec<(i32, i32, i32, i32, u32)> {
		let Some(data) = self.data.as_ref() else {
			return Vec::new();
		};

		let mut ret = Vec::new();
		ret.push((0, 0, data.0, data.1, self.object_id));

		for i in &self.children {
			let sub_surface = client.get_object::<wl::SubSurface>(*i).unwrap();
			let surface = client
				.get_object::<wl::Surface>(sub_surface.surface)
				.unwrap();

			let position = sub_surface.position;

			ret.extend(
				surface
					.get_front_buffers(client)
					.into_iter()
					.map(|x| (x.0 + position.0, x.1 + position.1, x.2, x.3, x.4)),
			);
		}

		ret
	}

	pub fn frame(&mut self, ms: u32, client: &mut wl::Client) -> Result<()> {
		for &callback in &self.current_frame_callbacks {
			client.send_message(wlm::Message {
				object_id: callback,
				op: 0,
				args: ms,
			})?;

			client.queue_remove_object(callback);
		}

		self.current_frame_callbacks.clear();

		for i in &self.children {
			let sub_surface = client.get_object::<wl::SubSurface>(*i)?;
			let surface = client.get_object_mut::<wl::Surface>(sub_surface.surface)?;

			surface.frame(ms, client)?;
		}

		Ok(())
	}

	pub fn wgpu_do_textures(
		&mut self,
		client: &mut wl::Client,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		sampler: &wgpu::Sampler,
		bind_group_layout: &wgpu::BindGroupLayout,
	) -> Result<()> {
		let Some(buffer_id) = self.current_buffer else {
			return Ok(());
		};

		let buffer = client.get_object_mut::<wl::Buffer>(buffer_id)?;

		if let Some((width, height, ..)) = &self.data {
			assert!(buffer.width == *width && buffer.height == *height);
		}

		let (_, _, (texture, _)) = self.data.get_or_insert_with(|| {
			let texture = device.create_texture(&wgpu::TextureDescriptor {
				size: wgpu::Extent3d {
					width: buffer.width as _,
					height: buffer.height as _,
					depth_or_array_layers: 1,
				},
				mip_level_count: 1,
				sample_count: 1,
				dimension: wgpu::TextureDimension::D2,
				format: wgpu::TextureFormat::Bgra8UnormSrgb,
				usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
				label: None,
				view_formats: &[],
			});

			let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(
							&texture.create_view(&wgpu::TextureViewDescriptor::default()),
						),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(sampler),
					},
				],
				label: None,
			});

			(buffer.width, buffer.height, (texture, bind_group))
		});

		buffer.wgpu_get_pixels(queue, texture);

		buffer.release(client)?;
		self.current_buffer = None;

		for i in &self.children {
			let sub_surface = client.get_object::<wl::SubSurface>(*i)?;
			let surface = client.get_object_mut::<wl::Surface>(sub_surface.surface)?;

			surface.wgpu_do_textures(client, device, queue, sampler, bind_group_layout)?;
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
				self.pending_frame_callbacks.push(callback);
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

				self.current_frame_callbacks
					.extend(&self.pending_frame_callbacks);

				self.pending_frame_callbacks.clear();
			}
			7 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_buffer_transform
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
