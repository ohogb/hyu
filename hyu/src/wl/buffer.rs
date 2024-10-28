use glow::HasContext;

use crate::{Client, Point, Result, egl, wl};

pub enum BufferStorage {
	Shm {
		map: wl::SharedMap,
		offset: i32,
		stride: i32,
		format: u32,
	},
	Dmabuf {
		image: egl::Image,
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

	pub fn gl_get_pixels(
		&self,
		_client: &Client,
		glow: &glow::Context,
		texture: glow::NativeTexture,
	) -> Result<()> {
		match &self.storage {
			BufferStorage::Shm {
				map,
				offset,
				stride,
				..
			} => {
				let map = unsafe { (*map.as_mut_ptr()).as_slice() };

				let start = *offset as usize;
				let end = start + (stride * self.size.1) as usize;

				let buffer = &map[start..end];

				unsafe {
					glow.bind_texture(glow::TEXTURE_2D, Some(texture));

					glow.tex_parameter_i32(
						glow::TEXTURE_2D,
						glow::TEXTURE_MIN_FILTER,
						glow::LINEAR as _,
					);

					glow.tex_parameter_i32(
						glow::TEXTURE_2D,
						glow::TEXTURE_MAG_FILTER,
						glow::LINEAR as _,
					);

					glow.tex_image_2d(
						glow::TEXTURE_2D,
						0,
						glow::RGBA as _,
						self.size.0,
						self.size.1,
						0,
						glow::BGRA,
						glow::UNSIGNED_BYTE,
						Some(buffer),
					);

					glow.bind_texture(glow::TEXTURE_2D, None);
				};
			}
			BufferStorage::Dmabuf { image } => unsafe {
				glow.active_texture(glow::TEXTURE0);
				glow.bind_texture(glow::TEXTURE_2D, Some(texture));

				glow.tex_parameter_i32(
					glow::TEXTURE_2D,
					glow::TEXTURE_MIN_FILTER,
					glow::LINEAR as _,
				);

				glow.tex_parameter_i32(
					glow::TEXTURE_2D,
					glow::TEXTURE_MAG_FILTER,
					glow::LINEAR as _,
				);

				image.target_texture_2d_oes(glow::TEXTURE_2D as _);
				glow.bind_texture(glow::TEXTURE_2D, None);
			},
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
	fn handle(&mut self, client: &mut Client, op: u16, _params: &[u8]) -> Result<()> {
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
