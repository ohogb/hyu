use crate::{wl, Point, Result};

struct Ptr(std::ptr::NonNull<std::ffi::c_void>);

unsafe impl Send for Ptr {}
unsafe impl Sync for Ptr {}

pub struct Map {
	ptr: Ptr,
	size: usize,
}

impl Map {
	pub fn new(size: usize, fd: std::os::fd::RawFd) -> Result<Self> {
		Ok(Self {
			ptr: Ptr(unsafe {
				nix::sys::mman::mmap(
					None,
					std::num::NonZeroUsize::new(size).unwrap(),
					nix::sys::mman::ProtFlags::PROT_READ | nix::sys::mman::ProtFlags::PROT_WRITE,
					nix::sys::mman::MapFlags::MAP_SHARED,
					std::os::fd::BorrowedFd::borrow_raw(fd),
					0,
				)?
			}),
			size,
		})
	}

	pub fn as_slice(&self) -> &[u8] {
		unsafe { std::slice::from_raw_parts(self.ptr.0.as_ptr() as *const u8, self.size) }
	}
}

impl Drop for Map {
	fn drop(&mut self) {
		unsafe { nix::sys::mman::munmap(self.ptr.0, self.size).unwrap() };
	}
}

#[derive(Clone)]
pub struct SharedMap(std::sync::Arc<std::cell::SyncUnsafeCell<Map>>);

impl SharedMap {
	pub fn new(map: Map) -> Self {
		Self(std::sync::Arc::new(std::cell::SyncUnsafeCell::new(map)))
	}

	pub fn get(&self) -> &mut Map {
		unsafe { &mut *self.0.get() }
	}
}

pub struct ShmPool {
	object_id: wl::Id<Self>,
	fd: std::os::fd::RawFd,
	size: u32,
	map: SharedMap,
}

impl ShmPool {
	pub fn new(object_id: wl::Id<Self>, fd: std::os::fd::RawFd, size: u32) -> Result<Self> {
		Ok(Self {
			object_id,
			fd,
			size,
			map: SharedMap::new(Map::new(size as _, fd)?),
		})
	}

	pub fn get_map(&self) -> &[u8] {
		self.map.get().as_slice()
	}
}

impl wl::Object for ShmPool {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:create_buffer
				let (id, offset, width, height, stride, format): (
					wl::Id<wl::Buffer>,
					i32,
					i32,
					i32,
					i32,
					u32,
				) = wlm::decode::from_slice(params)?;

				client.new_object(
					id,
					wl::Buffer::new(
						id,
						Point(width, height),
						wl::BufferStorage::Shm {
							map: self.map.clone(),
							offset,
							stride,
							format,
						},
					),
				);
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:destroy
				client.remove_object(self.object_id)?;
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:resize
				let size: u32 = wlm::decode::from_slice(params)?;

				self.size = size;
				*self.map.get() = Map::new(size as _, self.fd)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ShmPool"),
		}

		Ok(())
	}
}
