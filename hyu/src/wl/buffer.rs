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

	pub fn get_pixels(&self) -> Vec<u8> {
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
			)
			.iter()
			.cloned()
			.collect::<Vec<_>>();

			nix::sys::mman::munmap(map, self.size as _).unwrap();
			ret
		}
	}
}

impl wl::Object for Buffer {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_buffer:request:destroy
			}
			_ => Err(format!("unknown op '{op}' in Buffer"))?,
		}

		Ok(())
	}
}
