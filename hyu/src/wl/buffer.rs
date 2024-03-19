use crate::{wl, Result};

pub struct Buffer {
	object_id: u32,
	fd: std::os::fd::RawFd,
	size: u32,
	offset: i32,
	pub width: i32,
	pub height: i32,
	pub stride: i32,
	format: u32,
}

impl Buffer {
	pub fn new(
		object_id: u32,
		fd: std::os::fd::RawFd,
		size: u32,
		offset: i32,
		width: i32,
		height: i32,
		stride: i32,
		format: u32,
	) -> Self {
		Self {
			object_id,
			fd,
			size,
			offset,
			width,
			height,
			stride,
			format,
		}
	}

	pub fn wgpu_get_pixels(&self, queue: &wgpu::Queue, texture: &wgpu::Texture) {
		unsafe {
			let map = nix::sys::mman::mmap(
				None,
				std::num::NonZeroUsize::new(self.size as _).unwrap(),
				nix::sys::mman::ProtFlags::PROT_READ | nix::sys::mman::ProtFlags::PROT_WRITE,
				nix::sys::mman::MapFlags::MAP_SHARED,
				Some(std::os::fd::BorrowedFd::borrow_raw(self.fd)),
				0,
			)
			.unwrap();

			let ret = std::slice::from_raw_parts(
				(map as *const u8).offset(self.offset as _),
				(self.stride * self.height) as _,
			);

			queue.write_texture(
				wgpu::ImageCopyTexture {
					texture,
					mip_level: 0,
					origin: wgpu::Origin3d::ZERO,
					aspect: wgpu::TextureAspect::All,
				},
				ret,
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

			nix::sys::mman::munmap(map, self.size as _).unwrap();
		}
	}

	pub fn release(&self, client: &mut wl::Client) -> Result<()> {
		client.send_message(wlm::Message {
			object_id: self.object_id,
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
