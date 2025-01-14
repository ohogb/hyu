use crate::{Client, Point, Result, renderer, state::HwState, wl};

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

	pub fn vk_copy_to_texture(
		&self,
		_client: &mut Client,
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

					let buffer_ptr = unsafe {
						vk.device.map_memory(
							buffer_device_memory,
							0,
							buffer_size as _,
							ash::vk::MemoryMapFlags::default(),
						)?
					};

					*texture = Some((
						self.size,
						wl::SurfaceTexture::Vk(renderer::vulkan::Texture {
							image,
							image_device_memory,
							image_view,
							image_layout: ash::vk::ImageLayout::UNDEFINED,
							buffer,
							buffer_device_memory,
							buffer_size,
							buffer_ptr,
						}),
					))
				}

				let Some((_, wl::SurfaceTexture::Vk(texture))) = texture else {
					panic!();
				};

				let start = *offset as usize;
				let end = start + (stride * self.size.1) as usize;

				let map = unsafe { (*map.as_mut_ptr()).as_slice() };
				let buffer = &map[start..end];

				assert!(buffer.len() <= texture.buffer_size);

				unsafe {
					std::ptr::copy(buffer.as_ptr(), texture.buffer_ptr as *mut u8, buffer.len());
				}

				renderer::vulkan::Renderer::single_time_command(
					&vk.device,
					vk.queue,
					vk.command_pool,
					|command_buffer| {
						let range = ash::vk::ImageSubresourceRange::default()
							.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
							.base_mip_level(0)
							.level_count(1)
							.base_array_layer(0)
							.layer_count(1);

						let barrier = ash::vk::ImageMemoryBarrier::default()
							.src_access_mask(ash::vk::AccessFlags::SHADER_READ)
							.dst_access_mask(ash::vk::AccessFlags::TRANSFER_WRITE)
							.old_layout(texture.image_layout)
							.new_layout(ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL)
							.src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
							.dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
							.image(texture.image)
							.subresource_range(range);

						let barriers = [barrier];

						unsafe {
							vk.device.cmd_pipeline_barrier(
								command_buffer,
								ash::vk::PipelineStageFlags::FRAGMENT_SHADER,
								ash::vk::PipelineStageFlags::TRANSFER,
								ash::vk::DependencyFlags::empty(),
								&[],
								&[],
								&barriers,
							);
						}

						let regions = [ash::vk::BufferImageCopy::default()
							.buffer_offset(0)
							.buffer_row_length(0)
							.buffer_image_height(0)
							.image_subresource(
								ash::vk::ImageSubresourceLayers::default()
									.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
									.mip_level(0)
									.base_array_layer(0)
									.layer_count(1),
							)
							.image_offset(ash::vk::Offset3D::default())
							.image_extent(
								ash::vk::Extent3D::default()
									.width(self.size.0 as _)
									.height(self.size.1 as _)
									.depth(1),
							)];

						unsafe {
							vk.device.cmd_copy_buffer_to_image(
								command_buffer,
								texture.buffer,
								texture.image,
								ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
								&regions,
							);
						}

						let barrier = ash::vk::ImageMemoryBarrier::default()
							.src_access_mask(ash::vk::AccessFlags::TRANSFER_WRITE)
							.dst_access_mask(ash::vk::AccessFlags::SHADER_READ)
							.old_layout(ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL)
							.new_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
							.src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
							.dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
							.image(texture.image)
							.subresource_range(range);

						let barriers = [barrier];

						unsafe {
							vk.device.cmd_pipeline_barrier(
								command_buffer,
								ash::vk::PipelineStageFlags::TRANSFER,
								ash::vk::PipelineStageFlags::ALL_GRAPHICS,
								ash::vk::DependencyFlags::empty(),
								&[],
								&[],
								&barriers,
							);
						}

						texture.image_layout = ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
						Ok(())
					},
				)?;
			}
			BufferStorage::Dmabuf { image, image_view } => {
				if texture.is_none() {
					*texture = Some((
						self.size,
						wl::SurfaceTexture::Vk(renderer::vulkan::Texture {
							image: *image,
							image_device_memory: ash::vk::DeviceMemory::null(),
							image_view: *image_view,
							image_layout: ash::vk::ImageLayout::UNDEFINED,
							buffer: ash::vk::Buffer::null(),
							buffer_device_memory: ash::vk::DeviceMemory::null(),
							buffer_size: 0,
							buffer_ptr: std::ptr::null_mut(),
						}),
					))
				}

				let Some((_, wl::SurfaceTexture::Vk(texture))) = texture else {
					panic!();
				};

				// so hacky
				if texture.image != *image {
					*texture = renderer::vulkan::Texture {
						image: *image,
						image_device_memory: ash::vk::DeviceMemory::null(),
						image_view: *image_view,
						image_layout: ash::vk::ImageLayout::UNDEFINED,
						buffer: ash::vk::Buffer::null(),
						buffer_device_memory: ash::vk::DeviceMemory::null(),
						buffer_size: 0,
						buffer_ptr: std::ptr::null_mut(),
					};
				}
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
