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

					*texture = Some((
						self.size,
						wl::SurfaceTexture::Vk(renderer::vulkan::Texture {
							image,
							image_device_memory,
							image_view,
							buffer,
							buffer_device_memory,
							buffer_size,
						}),
					))
				}

				let Some((_, wl::SurfaceTexture::Vk(texture))) = texture else {
					panic!();
				};

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
			}
			BufferStorage::Dmabuf { image, image_view } => {
				if texture.is_none() {
					*texture = Some((
						self.size,
						wl::SurfaceTexture::Vk(renderer::vulkan::Texture {
							image: *image,
							image_device_memory: ash::vk::DeviceMemory::null(),
							image_view: *image_view,
							buffer: ash::vk::Buffer::null(),
							buffer_device_memory: ash::vk::DeviceMemory::null(),
							buffer_size: 0,
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
						buffer: ash::vk::Buffer::null(),
						buffer_device_memory: ash::vk::DeviceMemory::null(),
						buffer_size: 0,
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
