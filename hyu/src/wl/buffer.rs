use glow::HasContext;

use crate::{Client, Point, Result, egl, renderer, state::HwState, wl};

pub enum BufferStorage {
	Shm {
		map: wl::SharedMap,
		offset: i32,
		stride: i32,
		format: u32,
	},
	Dmabuf {
		image: ash::vk::Image,
		image_view: ash::vk::ImageView,
	},
}

pub struct Buffer {
	object_id: wl::Id<Self>,
	pub size: Point,
	pub storage: BufferStorage,
}

impl Buffer {
	pub fn new(object_id: wl::Id<Self>, size: Point, storage: BufferStorage) -> Self {
		Self {
			object_id,
			size,
			storage,
		}
	}

	// pub fn gl_get_pixels(
	// 	&self,
	// 	_client: &Client,
	// 	glow: &glow::Context,
	// 	texture: glow::NativeTexture,
	// ) -> Result<()> {
	// 	match &self.storage {
	// 		BufferStorage::Shm {
	// 			map,
	// 			offset,
	// 			stride,
	// 			..
	// 		} => {
	// 			let map = unsafe { (*map.as_mut_ptr()).as_slice() };
	//
	// 			let start = *offset as usize;
	// 			let end = start + (stride * self.size.1) as usize;
	//
	// 			let buffer = &map[start..end];
	//
	// 			unsafe {
	// 				glow.bind_texture(glow::TEXTURE_2D, Some(texture));
	//
	// 				glow.tex_parameter_i32(
	// 					glow::TEXTURE_2D,
	// 					glow::TEXTURE_MIN_FILTER,
	// 					glow::LINEAR as _,
	// 				);
	//
	// 				glow.tex_parameter_i32(
	// 					glow::TEXTURE_2D,
	// 					glow::TEXTURE_MAG_FILTER,
	// 					glow::LINEAR as _,
	// 				);
	//
	// 				glow.tex_image_2d(
	// 					glow::TEXTURE_2D,
	// 					0,
	// 					glow::RGBA as _,
	// 					self.size.0,
	// 					self.size.1,
	// 					0,
	// 					glow::BGRA,
	// 					glow::UNSIGNED_BYTE,
	// 					Some(buffer),
	// 				);
	//
	// 				glow.bind_texture(glow::TEXTURE_2D, None);
	// 			};
	// 		}
	// 		BufferStorage::Dmabuf { image } => unsafe {
	// 			glow.active_texture(glow::TEXTURE0);
	// 			glow.bind_texture(glow::TEXTURE_2D, Some(texture));
	//
	// 			glow.tex_parameter_i32(
	// 				glow::TEXTURE_2D,
	// 				glow::TEXTURE_MIN_FILTER,
	// 				glow::LINEAR as _,
	// 			);
	//
	// 			glow.tex_parameter_i32(
	// 				glow::TEXTURE_2D,
	// 				glow::TEXTURE_MAG_FILTER,
	// 				glow::LINEAR as _,
	// 			);
	//
	// 			image.target_texture_2d_oes(glow::TEXTURE_2D as _);
	// 			glow.bind_texture(glow::TEXTURE_2D, None);
	// 		},
	// 	}
	//
	// 	Ok(())
	// }

	pub fn vk_copy_to_texture(
		&self,
		client: &mut Client,
		vk: &mut renderer::vulkan::Renderer,
		texture: &mut Option<(Point, wl::SurfaceTexture)>,
	) -> Result<()> {
		match &self.storage {
			BufferStorage::Shm {
				map,
				offset,
				stride,
				..
			} => {
				if texture.is_none() {
					let (image, image_device_memory, image_view) =
						renderer::vulkan::Renderer::create_image(
							&vk.device,
							&vk.instance,
							vk.physical_device,
							self.size.0 as _,
							self.size.1 as _,
							self.size.0 as usize * 4,
						)?;

					let buffer_size = (self.size.0 * self.size.1 * 4) as usize;

					let (buffer, buffer_device_memory) = renderer::vulkan::Renderer::create_buffer(
						&vk.instance,
						&vk.device,
						vk.physical_device,
						buffer_size,
						ash::vk::BufferUsageFlags::TRANSFER_SRC,
						ash::vk::MemoryPropertyFlags::HOST_VISIBLE
							| ash::vk::MemoryPropertyFlags::HOST_COHERENT,
					)?;

					let descriptor_set = {
						let descriptor_set_allocate_info =
							ash::vk::DescriptorSetAllocateInfo::default()
								.descriptor_pool(vk.descriptor_pool)
								.set_layouts(&vk.descriptor_set_layouts);

						let descriptor_set = unsafe {
							vk.device
								.allocate_descriptor_sets(&descriptor_set_allocate_info)?
						}
						.into_iter()
						.next()
						.unwrap();

						let descriptor_image_infos = [ash::vk::DescriptorImageInfo::default()
							.image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
							.image_view(image_view)
							.sampler(vk.sampler)];

						let descriptor_writes = [ash::vk::WriteDescriptorSet::default()
							.dst_set(descriptor_set)
							.dst_binding(0)
							.descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
							.descriptor_count(1)
							.image_info(&descriptor_image_infos)];

						unsafe {
							vk.device.update_descriptor_sets(&descriptor_writes, &[]);
						}

						descriptor_set
					};

					*texture = Some((
						self.size,
						wl::SurfaceTexture::Vk(renderer::vulkan::Texture {
							image,
							image_device_memory,
							image_view,
							buffer,
							buffer_device_memory,
							buffer_size,
							descriptor_set,
						}),
					))
				}

				let Some((_, wl::SurfaceTexture::Vk(texture))) = texture else {
					panic!();
				};

				// renderer::vulkan::Renderer::clear_image(
				// 	&vk.device,
				// 	vk.queue,
				// 	vk.command_pool,
				// 	texture.image,
				// 	(0.0, 0.0, 0.5, 1.0),
				// )?;

				renderer::vulkan::Renderer::transition_image_layout(
					&vk.device,
					vk.queue,
					vk.command_pool,
					texture.image,
					ash::vk::ImageLayout::UNDEFINED,
					ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
				)?;

				let start = *offset as usize;
				let end = start + (stride * self.size.1) as usize;

				let map = unsafe { (*map.as_mut_ptr()).as_slice() };
				let buffer = &map[start..end];

				renderer::vulkan::Renderer::copy_to_buffer(
					&vk.device,
					texture.buffer_device_memory,
					texture.buffer_size,
					buffer,
				)?;

				renderer::vulkan::Renderer::copy_buffer_to_image(
					&vk.device,
					vk.queue,
					vk.command_pool,
					texture.buffer,
					texture.image,
					self.size.0 as _,
					self.size.1 as _,
				)?;

				renderer::vulkan::Renderer::transition_image_layout(
					&vk.device,
					vk.queue,
					vk.command_pool,
					texture.image,
					ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
					ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
				)?;

				// println!("HEREEEEEEE");
			}
			BufferStorage::Dmabuf { image, image_view } => {
				if texture.is_none() {
					// let (image, image_device_memory, image_view) =
					// 	renderer::vulkan::Renderer::create_image(
					// 		&vk.device,
					// 		&vk.instance,
					// 		vk.physical_device,
					// 		self.size.0 as _,
					// 		self.size.1 as _,
					// 		self.size.0 as usize * 4,
					// 	)?;
					println!("HERE");

					let descriptor_set = {
						let descriptor_set_allocate_info =
							ash::vk::DescriptorSetAllocateInfo::default()
								.descriptor_pool(vk.descriptor_pool)
								.set_layouts(&vk.descriptor_set_layouts);

						let descriptor_set = unsafe {
							vk.device
								.allocate_descriptor_sets(&descriptor_set_allocate_info)?
						}
						.into_iter()
						.next()
						.unwrap();

						let descriptor_image_infos = [ash::vk::DescriptorImageInfo::default()
							.image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
							.image_view(*image_view)
							.sampler(vk.sampler)];

						let descriptor_writes = [ash::vk::WriteDescriptorSet::default()
							.dst_set(descriptor_set)
							.dst_binding(0)
							.descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
							.descriptor_count(1)
							.image_info(&descriptor_image_infos)];

						unsafe {
							vk.device.update_descriptor_sets(&descriptor_writes, &[]);
						}

						descriptor_set
					};

					*texture = Some((
						self.size,
						wl::SurfaceTexture::Vk(renderer::vulkan::Texture {
							image: *image,
							image_device_memory: ash::vk::DeviceMemory::null(),
							image_view: *image_view,
							buffer: ash::vk::Buffer::null(),
							buffer_device_memory: ash::vk::DeviceMemory::null(),
							buffer_size: 0,
							descriptor_set,
						}),
					))
				}

				let Some((_, wl::SurfaceTexture::Vk(texture))) = texture else {
					panic!();
				};

				// texture.image = image.clone();
				// texture.image_view = image_view.clone();

				// renderer::vulkan::Renderer::single_time_command(
				// 	&vk.device,
				// 	vk.queue,
				// 	vk.command_pool,
				// 	|command_buffer| {
				// 		// asdf
				//                     vk.device.cmd_copy_image(command_buffer, image, )
				// 		Ok(())
				// 	},
				// )?;
			}
		}

		Ok(())
	}

	pub fn release(&self, client: &mut Client) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_buffer:event:release
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (),
		})
	}
}

impl wl::Object for Buffer {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		_params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_buffer:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Buffer"),
		}

		Ok(())
	}
}
