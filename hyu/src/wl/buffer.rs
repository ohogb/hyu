use glow::HasContext;

use crate::{wl, Result};

pub enum BufferStorage {
	Shm {
		pool_id: wl::Id<wl::ShmPool>,
		offset: i32,
		stride: i32,
		format: u32,
	},
	Dmabuf {},
}

pub struct Buffer {
	object_id: wl::Id<Self>,
	pub width: i32,
	pub height: i32,
	pub storage: BufferStorage,
}

impl Buffer {
	pub fn new(object_id: wl::Id<Self>, width: i32, height: i32, storage: BufferStorage) -> Self {
		Self {
			object_id,
			width,
			height,
			storage,
		}
	}

	pub fn wgpu_get_pixels(
		&self,
		client: &wl::Client,
		queue: &wgpu::Queue,
		texture: &wgpu::Texture,
	) -> Result<()> {
		match self.storage {
			BufferStorage::Shm {
				pool_id,
				offset,
				stride,
				..
			} => {
				let pool = client.get_object(pool_id)?;
				let map = pool.get_map().ok_or("pool is not mapped")?;

				let start = offset as usize;
				let end = start + (stride * self.height) as usize;

				let buffer = &map[start..end];

				queue.write_texture(
					wgpu::ImageCopyTexture {
						texture,
						mip_level: 0,
						origin: wgpu::Origin3d::ZERO,
						aspect: wgpu::TextureAspect::All,
					},
					buffer,
					wgpu::ImageDataLayout {
						offset: 0,
						bytes_per_row: Some(stride as _),
						rows_per_image: Some(self.height as _),
					},
					wgpu::Extent3d {
						width: self.width as _,
						height: self.height as _,
						depth_or_array_layers: 1,
					},
				);
			}
			BufferStorage::Dmabuf {} => todo!(),
		}

		Ok(())
	}

	pub fn gl_get_pixels(
		&self,
		client: &wl::Client,
		glow: &glow::Context,
		texture: glow::NativeTexture,
	) -> Result<()> {
		match self.storage {
			BufferStorage::Shm {
				pool_id,
				offset,
				stride,
				..
			} => {
				let pool = client.get_object(pool_id)?;
				let map = pool.get_map().ok_or("pool is not mapped")?;

				let start = offset as usize;
				let end = start + (stride * self.height) as usize;

				let buffer = &map[start..end];

				unsafe {
					glow.bind_texture(glow::TEXTURE_2D, Some(texture));

					glow.tex_image_2d(
						glow::TEXTURE_2D,
						0,
						glow::RGBA as _,
						self.width,
						self.height,
						0,
						glow::BGRA,
						glow::UNSIGNED_BYTE,
						Some(buffer),
					);

					glow.generate_mipmap(glow::TEXTURE_2D);
				};
			}
			BufferStorage::Dmabuf {} => todo!(),
		}

		Ok(())
	}

	pub fn release(&self, client: &mut wl::Client) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_buffer:event:release
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (),
		})
	}
}

impl wl::Object for Buffer {
	fn handle(&mut self, client: &mut wl::Client, op: u16, _params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_buffer:request:destroy
				client.remove_object(self.object_id)?;
			}
			_ => Err(format!("unknown op '{op}' in Buffer"))?,
		}

		Ok(())
	}
}
