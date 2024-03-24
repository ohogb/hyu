use crate::{wl, Result};

pub struct Buffer {
	object_id: wl::Id<Self>,
	pool_id: wl::Id<wl::ShmPool>,
	offset: i32,
	pub width: i32,
	pub height: i32,
	pub stride: i32,
	format: u32,
}

impl Buffer {
	pub fn new(
		object_id: wl::Id<Self>,
		pool_id: wl::Id<wl::ShmPool>,
		offset: i32,
		width: i32,
		height: i32,
		stride: i32,
		format: u32,
	) -> Self {
		Self {
			object_id,
			pool_id,
			offset,
			width,
			height,
			stride,
			format,
		}
	}

	pub fn wgpu_get_pixels(
		&self,
		client: &wl::Client,
		queue: &wgpu::Queue,
		texture: &wgpu::Texture,
	) -> Result<()> {
		let pool = client.get_object(self.pool_id)?;
		let map = pool.get_map().ok_or("pool is not mapped")?;

		let start = self.offset as usize;
		let end = start + (self.stride * self.height) as usize;

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
				bytes_per_row: Some(self.stride as _),
				rows_per_image: Some(self.height as _),
			},
			wgpu::Extent3d {
				width: self.width as _,
				height: self.height as _,
				depth_or_array_layers: 1,
			},
		);

		Ok(())
	}

	pub fn release(&self, client: &mut wl::Client) -> Result<()> {
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (),
		})?;

		Ok(())
	}
}

impl wl::Object for Buffer {
	fn handle(&mut self, client: &mut wl::Client, op: u16, _params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_buffer:request:destroy
				client.queue_remove_object(self.object_id);
			}
			_ => Err(format!("unknown op '{op}' in Buffer"))?,
		}

		Ok(())
	}
}
