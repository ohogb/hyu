use crate::backend::drm::gbm;

#[link(name = "gbm")]
extern "C" {
	fn gbm_surface_lock_front_buffer(surface: u64) -> u64;
	fn gbm_surface_release_buffer(surface: u64, bo: u64);
}

#[repr(transparent)]
pub struct Surface {
	ptr: std::ptr::NonNull<()>,
}

impl Surface {
	pub fn as_ptr(&self) -> u64 {
		self.ptr.as_ptr() as _
	}

	pub fn lock_front_buffer(&self) -> Option<gbm::BufferObject> {
		unsafe { std::mem::transmute(gbm_surface_lock_front_buffer(self.as_ptr())) }
	}

	pub fn release_buffer(&self, buffer: gbm::BufferObject) {
		unsafe {
			gbm_surface_release_buffer(self.as_ptr(), buffer.as_ptr());
		}
	}
}
