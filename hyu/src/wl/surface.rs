use glow::HasContext;

use crate::{state, wl, Result};

pub enum SubSurfaceMode {
	Sync,
	Desync,
}

pub enum SurfaceRole {
	XdgToplevel,
	XdgPopup,
	SubSurface {
		mode: SubSurfaceMode,
		parent: wl::Id<wl::Surface>,
	},
	Cursor,
}

pub enum SurfaceTexture {
	Wgpu(wgpu::Texture, wgpu::BindGroup),
	Gl(glow::NativeTexture),
}

pub struct Surface {
	pub object_id: wl::Id<Self>,
	pub children: Vec<wl::Id<wl::SubSurface>>,
	pending_buffer: Option<wl::Id<wl::Buffer>>,
	current_buffer: Option<wl::Id<wl::Buffer>>,
	pending_frame_callbacks: Vec<wl::Id<wl::Callback>>,
	current_frame_callbacks: Vec<wl::Id<wl::Callback>>,
	pending_input_region: Option<wl::Region>,
	pub current_input_region: Option<wl::Region>,
	pub data: Option<(i32, i32, SurfaceTexture)>,
	pub role: Option<SurfaceRole>,
}

impl Surface {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self {
			object_id,
			children: Vec::new(),
			pending_buffer: None,
			current_buffer: None,
			pending_frame_callbacks: Vec::new(),
			current_frame_callbacks: Vec::new(),
			pending_input_region: None,
			current_input_region: None,
			data: None,
			role: None,
		}
	}

	pub fn push(&mut self, child: wl::Id<wl::SubSurface>) {
		self.children.push(child);
	}

	pub fn get_front_buffers(
		&self,
		client: &wl::Client,
	) -> Vec<(i32, i32, i32, i32, wl::Id<wl::Surface>)> {
		let Some(data) = self.data.as_ref() else {
			return Vec::new();
		};

		let mut ret = Vec::new();
		ret.push((0, 0, data.0, data.1, self.object_id));

		for i in &self.children {
			let sub_surface = client.get_object(*i).unwrap();
			let surface = client.get_object(sub_surface.surface).unwrap();

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
			let callback = client.get_object(callback)?.clone();
			callback.done(client, ms)?;
		}

		self.current_frame_callbacks.clear();

		for i in &self.children {
			let sub_surface = client.get_object(*i)?;
			let surface = client.get_object_mut(sub_surface.surface)?;

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
		if let Some(buffer_id) = self.current_buffer {
			let buffer = client.get_object_mut(buffer_id)?;

			if let Some((width, height, ..)) = &self.data {
				assert!(buffer.width == *width && buffer.height == *height);
			}

			let (_, _, texture) = self.data.get_or_insert_with(|| {
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

				(
					buffer.width,
					buffer.height,
					SurfaceTexture::Wgpu(texture, bind_group),
				)
			});

			let SurfaceTexture::Wgpu(texture, _) = texture else {
				panic!();
			};

			buffer.wgpu_get_pixels(client, queue, texture)?;

			buffer.release(client)?;
			self.current_buffer = None;
		}

		for i in &self.children {
			let sub_surface = client.get_object(*i)?;
			let surface = client.get_object_mut(sub_surface.surface)?;

			surface.wgpu_do_textures(client, device, queue, sampler, bind_group_layout)?;
		}

		Ok(())
	}

	pub fn gl_do_textures(&mut self, client: &mut wl::Client, glow: &glow::Context) -> Result<()> {
		if let Some(buffer_id) = self.current_buffer {
			let buffer = client.get_object_mut(buffer_id)?;

			if let Some((width, height, tex)) = &self.data {
				if buffer.width != *width || buffer.height != *height {
					let SurfaceTexture::Gl(tex) = tex else {
						unreachable!();
					};

					unsafe {
						glow.delete_texture(*tex);
					}

					self.data = None;
				}
			}

			let (_, _, texture) = self.data.get_or_insert_with(|| {
				let texture = unsafe { glow.create_texture().unwrap() };
				(buffer.width, buffer.height, SurfaceTexture::Gl(texture))
			});

			let SurfaceTexture::Gl(texture) = texture else {
				panic!();
			};

			buffer.gl_get_pixels(client, glow, *texture)?;

			buffer.release(client)?;
			self.current_buffer = None;
		}

		for i in &self.children {
			let sub_surface = client.get_object(*i)?;
			let surface = client.get_object_mut(sub_surface.surface)?;

			surface.gl_do_textures(client, glow)?;
		}

		Ok(())
	}

	pub fn set_role(&mut self, role: SurfaceRole) -> Result<()> {
		if self.role.is_some() {
			Err("surface already has a role")?;
		}

		self.role = Some(role);
		Ok(())
	}

	// https://wayland.app/protocols/wayland#wl_surface:request:commit
	pub fn commit(&mut self) -> Result<()> {
		if let Some(buffer_id) = self.pending_buffer {
			self.current_buffer = Some(buffer_id);
			self.pending_buffer = None;
		}

		self.current_frame_callbacks
			.extend(&self.pending_frame_callbacks);

		self.pending_frame_callbacks.clear();

		if let Some(region) = &self.pending_input_region {
			self.current_input_region = Some(region.clone());
			self.pending_input_region = None;
		}

		Ok(())
	}
}

impl wl::Object for Surface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:destroy
				state::CHANGES
					.lock()
					.unwrap()
					.push(state::Change::RemoveSurface(client.fd, self.object_id));
				client.remove_object(self.object_id)?;
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:attach
				let (buffer, x, y): (wl::Id<wl::Buffer>, u32, u32) =
					wlm::decode::from_slice(params)?;

				assert!(x == 0);
				assert!(y == 0);

				self.pending_buffer = if !buffer.is_null() {
					Some(buffer)
				} else {
					None
				};
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:damage
			}
			3 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:frame
				let callback: wl::Id<wl::Callback> = wlm::decode::from_slice(params)?;
				client.new_object(callback, wl::Callback::new(callback));

				self.pending_frame_callbacks.push(callback);
			}
			4 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_opaque_region
			}
			5 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_input_region
				let region: wl::Id<wl::Region> = wlm::decode::from_slice(params)?;

				let region = if region.is_null() {
					wl::Region::new(wl::Id::null())
				} else {
					client.get_object(region)?.clone()
				};

				self.pending_input_region = Some(region);
			}
			6 => {
				self.commit()?;
			}
			7 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_buffer_transform
			}
			8 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_buffer_scale
				let _scale: u32 = wlm::decode::from_slice(params)?;
			}
			9 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:damage_buffer
				let (_x, _y, _width, _height): (u32, u32, u32, u32) =
					wlm::decode::from_slice(params)?;
			}
			_ => Err(format!("unknown op '{op}' in Surface"))?,
		}

		Ok(())
	}
}
