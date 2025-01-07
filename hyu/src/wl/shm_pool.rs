use crate::{Client, Point, Result, state::HwState, wl};

struct Ptr(std::ptr::NonNull<std::ffi::c_void>);

unsafe impl Send for Ptr {}
unsafe impl Sync for Ptr {}

pub struct Map {
	ptr: Ptr,
	size: usize,
}

impl Map {
	pub fn new(size: usize, fd: std::os::fd::RawFd) -> Result<Self> {
		let ptr = unsafe {
			nix::sys::mman::mmap(
				None,
				std::num::NonZeroUsize::new(size).unwrap(),
				nix::sys::mman::ProtFlags::PROT_READ,
				nix::sys::mman::MapFlags::MAP_SHARED,
				std::os::fd::BorrowedFd::borrow_raw(fd),
				0,
			)?
		};

		nix::unistd::close(fd)?;

		Ok(Self {
			ptr: Ptr(ptr),
			size,
		})
	}

	pub fn remap(&mut self, size: usize) -> Result<()> {
		let ptr = unsafe {
			nix::sys::mman::mremap(
				self.ptr.0,
				self.size,
				size,
				nix::sys::mman::MRemapFlags::MREMAP_MAYMOVE,
				None,
			)?
		};

		self.ptr = Ptr(ptr);
		self.size = size;

		Ok(())
	}

	pub fn as_slice(&self) -> &[u8] {
		unsafe { std::slice::from_raw_parts(self.ptr.0.as_ptr() as *const u8, self.size) }
	}
}

impl Drop for Map {
	fn drop(&mut self) {
		unsafe {
			nix::sys::mman::munmap(self.ptr.0, self.size).unwrap();
		}
	}
}

#[derive(Clone)]
pub struct SharedMap(std::sync::Arc<std::cell::SyncUnsafeCell<Map>>);

impl SharedMap {
	pub fn new(map: Map) -> Self {
		Self(std::sync::Arc::new(std::cell::SyncUnsafeCell::new(map)))
	}

	pub fn as_mut_ptr(&self) -> *mut Map {
		self.0.get()
	}
}

pub struct ShmPool {
	object_id: wl::Id<Self>,
	map: SharedMap,
}

impl ShmPool {
	pub fn new(object_id: wl::Id<Self>, fd: std::os::fd::RawFd, size: u32) -> Result<Self> {
		Ok(Self {
			object_id,
			map: SharedMap::new(Map::new(size as _, fd)?),
		})
	}

	pub fn get_map(&self) -> &[u8] {
		unsafe { (*self.map.as_mut_ptr()).as_slice() }
	}
}

impl wl::Object for ShmPool {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
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
					wl::Buffer::new(id, Point(width, height), wl::BufferStorage::Shm {
						map: self.map.clone(),
						offset,
						stride,
						format,
					}),
				);
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:resize
				let size: u32 = wlm::decode::from_slice(params)?;

				unsafe {
					(*self.map.as_mut_ptr()).remap(size as _)?;
				}
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ShmPool"),
		}

		Ok(())
	}
}
