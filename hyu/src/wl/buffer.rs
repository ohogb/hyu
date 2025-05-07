use std::rc::Rc;

use crate::{Client, Connection, Point, Result, gbm, renderer, state::HwState, wl};

pub struct ShmBackingStorage {
	pub size: Point,
	pub map: wl::SharedMap,
	pub offset: i32,
	pub stride: i32,
	pub format: u32,
}

impl ShmBackingStorage {
	pub fn copy_into_texture(
		&self,
		vk: &renderer::vulkan::Renderer,
		output: &mut Option<(Point, renderer::vulkan::Texture)>,
	) -> Result<()> {
		if output.is_none() {
			let (image, image_device_memory, image_view) =
				renderer::vulkan::Renderer::create_image(
					&vk.device,
					&vk.instance,
					vk.physical_device,
					self.size.0 as _,
					self.size.1 as _,
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

			*output = Some((
				self.size,
				renderer::vulkan::Texture {
					image,
					image_device_memory,
					image_view,
					image_layout: ash::vk::ImageLayout::UNDEFINED,
					buffer,
					buffer_device_memory,
					buffer_size,
					buffer_ptr,
				},
			))
		}

		let Some((texture_size, texture)) = output else {
			panic!();
		};

		assert!(self.size == *texture_size);

		let start = self.offset as usize;
		let end = start + (self.stride * self.size.1) as usize;

		let map = unsafe { (*self.map.as_mut_ptr()).as_slice() };
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
		)
	}
}

pub struct DmabufBackingStorage {
	pub size: Point,
	pub attributes: wl::DmabufAttributes,
	pub image: ash::vk::Image,
	pub image_view: ash::vk::ImageView,
	pub gbm_buffer_object: Option<gbm::BufferObject>,
}

pub enum BufferBackingStorage {
	Shm(ShmBackingStorage),
	Dmabuf(DmabufBackingStorage),
}

pub struct Buffer {
	object_id: wl::Id<Self>,
	conn: Rc<Connection>,
	pub backing_storage: BufferBackingStorage,
}

impl Buffer {
	pub fn new(
		object_id: wl::Id<Self>,
		conn: Rc<Connection>,
		backing_storage: BufferBackingStorage,
	) -> Self {
		Self {
			object_id,
			conn,
			backing_storage,
		}
	}

	pub fn release(&self) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_buffer:event:release
		self.conn.send_message(wlm::Message {
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
