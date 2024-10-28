use crate::gbm;

#[link(name = "gbm")]
unsafe extern "C" {
	fn gbm_create_device(fd: std::os::fd::RawFd) -> u64;
	fn gbm_surface_create(device: u64, width: u32, height: u32, format: u32, flags: u32) -> u64;
}

#[repr(transparent)]
pub struct Device {
	ptr: std::ptr::NonNull<()>,
}

impl Device {
	pub fn create(fd: std::os::fd::RawFd) -> Self {
		Self {
			ptr: unsafe { std::mem::transmute(gbm_create_device(fd)) },
		}
	}

	pub fn as_ptr(&self) -> u64 {
		self.ptr.as_ptr() as _
	}

	pub fn create_surface(
		&self,
		width: u32,
		height: u32,
		format: u32,
		flags: u32,
	) -> Option<gbm::Surface> {
		unsafe {
			std::mem::transmute(gbm_surface_create(
				self.as_ptr(),
				width,
				height,
				format,
				flags,
			))
		}
	}
}
