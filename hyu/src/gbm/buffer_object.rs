use crate::{Result, drm};

#[link(name = "gbm")]
unsafe extern "C" {
	fn gbm_bo_set_user_data(bo: u64, data: u64, destructor: u64);
	fn gbm_bo_get_user_data(bo: u64) -> u64;
	fn gbm_bo_get_width(bo: u64) -> u32;
	fn gbm_bo_get_height(bo: u64) -> u32;
	fn gbm_bo_get_stride(bo: u64) -> u32;
	fn gbm_bo_get_bpp(bo: u64) -> u32;
	fn gbm_bo_get_handle(bo: u64) -> u64;
}

pub struct UserData {
	fb: u32,
}

extern "C" fn user_data_destructor(_bo: BufferObject, _user_data: &mut UserData) {}

#[derive(Debug)]
#[repr(transparent)]
pub struct BufferObject {
	ptr: std::ptr::NonNull<()>,
}

impl BufferObject {
	pub fn as_ptr(&self) -> u64 {
		self.ptr.as_ptr() as _
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

		let user_data = Box::leak(Box::new(UserData {
			fb: drm_device.add_fb(
				self.get_width(),
				self.get_height(),
				24,
				self.get_bpp() as _,
				self.get_stride(),
				self.get_handle() as _,
			)?,
		}));

		unsafe {
			gbm_bo_set_user_data(
				self.as_ptr(),
				user_data as *mut _ as _,
				user_data_destructor as _,
			)
		};

		Ok(user_data.fb)
	}
}
