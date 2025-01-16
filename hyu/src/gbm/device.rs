use crate::gbm;

#[link(name = "gbm")]
unsafe extern "C" {
	fn gbm_create_device(fd: std::os::fd::RawFd) -> Option<Device>;
	fn gbm_device_destroy(device: usize);
	fn gbm_bo_create_with_modifiers2(
		device: usize,
		width: u32,
		height: u32,
		format: u32,
		modifiers: usize,
		count: u32,
		flags: u32,
	) -> Option<gbm::BufferObject>;
}

#[repr(transparent)]
pub struct Device {
	ptr: std::num::NonZeroUsize,
}

impl Device {
	pub fn create(fd: std::os::fd::RawFd) -> Option<Self> {
		unsafe { gbm_create_device(fd) }
	}

	pub fn as_ptr(&self) -> usize {
		self.ptr.get()
	}

	pub fn create_buffer_object(
		&self,
		width: u32,
		height: u32,
		format: u32,
		modifiers: &[u64],
		flags: u32,
	) -> Option<gbm::BufferObject> {
		unsafe {
			gbm_bo_create_with_modifiers2(
				self.as_ptr(),
				width,
				height,
				format,
				modifiers.as_ptr() as _,
				modifiers.len() as _,
				flags,
			)
		}
	}
}

impl Drop for Device {
	fn drop(&mut self) {
		unsafe {
			gbm_device_destroy(self.as_ptr());
		}
	}
}
