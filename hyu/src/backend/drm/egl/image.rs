#[link(name = "EGL")]
extern "C" {
	fn eglGetProcAddress(name: *const i8) -> usize;
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Image {
	ptr: std::num::NonZeroU64,
}

impl Image {
	pub fn target_texture_2d_oes(&self, target: i32) {
		static EGL_IMAGE_TARGET_TEXTURE_2D_OES: std::sync::LazyLock<extern "C" fn(i32, u64)> =
			std::sync::LazyLock::new(|| unsafe {
				std::mem::transmute(eglGetProcAddress(c"glEGLImageTargetTexture2DOES".as_ptr()))
			});

		EGL_IMAGE_TARGET_TEXTURE_2D_OES(target, self.as_ptr())
	}

	pub fn as_ptr(&self) -> u64 {
		self.ptr.get()
	}
}
