use crate::{Result, drm};

#[link(name = "gbm")]
unsafe extern "C" {
	fn gbm_bo_destroy(bo: usize);
	fn gbm_bo_set_user_data(bo: usize, data: usize, destructor: usize);
	fn gbm_bo_get_user_data(bo: usize) -> u64;
	fn gbm_bo_get_width(bo: usize) -> u32;
	fn gbm_bo_get_height(bo: usize) -> u32;
	fn gbm_bo_get_stride(bo: usize) -> u32;
	fn gbm_bo_get_bpp(bo: usize) -> u32;
	fn gbm_bo_get_handle(bo: usize) -> u64;
	fn gbm_bo_get_fd(bo: usize) -> std::os::fd::RawFd;
	fn gbm_bo_get_modifier(bo: usize) -> u64;
}

pub struct UserData {
	fb: u32,
}

extern "C" fn user_data_destructor(
	_bo: std::mem::ManuallyDrop<BufferObject>,
	user_data: *mut UserData,
) {
	let _ = unsafe { Box::from_raw(user_data) };
	// TODO: free fb?
}

#[derive(Debug)]
#[repr(transparent)]
pub struct BufferObject {
	ptr: std::num::NonZeroUsize,
}

impl BufferObject {
	pub fn as_ptr(&self) -> usize {
		self.ptr.get()
	}

	pub fn get_width(&self) -> u32 {
		unsafe { gbm_bo_get_width(self.as_ptr()) }
	}

	pub fn get_height(&self) -> u32 {
		unsafe { gbm_bo_get_height(self.as_ptr()) }
	}

	pub fn get_stride(&self) -> u32 {
		unsafe { gbm_bo_get_stride(self.as_ptr()) }
	}

	pub fn get_bpp(&self) -> u32 {
		unsafe { gbm_bo_get_bpp(self.as_ptr()) }
	}

	pub fn get_handle(&self) -> u64 {
		unsafe { gbm_bo_get_handle(self.as_ptr()) }
	}

	pub fn get_fb(&self, drm_device: &drm::Device) -> Result<u32> {
		let user_data = unsafe { gbm_bo_get_user_data(self.as_ptr()) } as *const UserData;

		if !user_data.is_null() {
			return Ok(unsafe { (*user_data).fb });
		}

		let fb = drm_device.add_fb(
			self.get_width(),
			self.get_height(),
			24,
			self.get_bpp() as _,
			self.get_stride(),
			self.get_handle() as _,
		)?;

		let user_data = Box::into_raw(Box::new(UserData { fb }));
		unsafe { gbm_bo_set_user_data(self.as_ptr(), user_data as _, user_data_destructor as _) };

		Ok(fb)
	}

	pub fn get_fd(&self) -> std::os::fd::RawFd {
		unsafe { gbm_bo_get_fd(self.as_ptr()) }
	}

	pub fn get_modifier(&self) -> u64 {
		unsafe { gbm_bo_get_modifier(self.as_ptr()) }
	}
}

impl Drop for BufferObject {
	fn drop(&mut self) {
		unsafe {
			gbm_bo_destroy(self.as_ptr());
		}
	}
}
