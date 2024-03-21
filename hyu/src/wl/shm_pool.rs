use crate::{wl, Result};

struct Ptr(*mut std::ffi::c_void);

unsafe impl Send for Ptr {}

pub struct ShmPool {
	object_id: u32,
	fd: std::os::fd::RawFd,
	size: u32,
	map: Option<(Ptr, usize)>,
}

impl ShmPool {
	pub fn new(object_id: u32, fd: std::os::fd::RawFd, size: u32) -> Result<Self> {
		let mut ret = Self {
			object_id,
			fd,
			size,
			map: None,
		};

		ret.remap()?;
		Ok(ret)
	}

	fn remap(&mut self) -> Result<()> {
		if let Some((map, size)) = &self.map {
			unsafe { nix::sys::mman::munmap(map.0, *size)? };
		}

		self.map = Some((
			Ptr(unsafe {
				nix::sys::mman::mmap(
					None,
					std::num::NonZeroUsize::new(self.size as _).unwrap(),
					nix::sys::mman::ProtFlags::PROT_READ | nix::sys::mman::ProtFlags::PROT_WRITE,
					nix::sys::mman::MapFlags::MAP_SHARED,
					Some(std::os::fd::BorrowedFd::borrow_raw(self.fd)),
					0,
				)?
			}),
			self.size as _,
		));

		Ok(())
	}

	pub fn get_map(&self) -> Option<&[u8]> {
		let Some((map, size)) = &self.map else {
			return None;
		};

		Some(unsafe { std::slice::from_raw_parts(map.0 as *const u8, *size) })
	}
}

impl wl::Object for ShmPool {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:create_buffer
				let (id, offset, width, height, stride, format): (u32, i32, i32, i32, i32, u32) =
					wlm::decode::from_slice(&params)?;

				client.queue_new_object(
					id,
					wl::Buffer::new(id, self.object_id, offset, width, height, stride, format),
				);
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:destroy
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:resize
				let size: u32 = wlm::decode::from_slice(&params)?;

				self.size = size;
				self.remap()?;
			}
			_ => Err(format!("unknown op '{op}' in ShmPool"))?,
		}

		Ok(())
	}
}
