use crate::Result;

static mut EGL_DISPLAY: usize = 0;
static mut EGL_CREATE_IMAGE_KHR: usize = 0;
static mut EGL_IMAGE_TARGET_TEXTURE_2D_OES: usize = 0;
static mut EGL_DEBUG_MESSAGE_CONTROL_KHR: usize = 0;

pub fn init(display: usize, get: impl Fn(&str) -> Result<usize>) -> Result<()> {
	unsafe {
		EGL_DISPLAY = display;

		EGL_CREATE_IMAGE_KHR = get("eglCreateImageKHR")?;
		EGL_IMAGE_TARGET_TEXTURE_2D_OES = get("glEGLImageTargetTexture2DOES")?;
		EGL_DEBUG_MESSAGE_CONTROL_KHR = get("eglDebugMessageControlKHR")?;

		extern "C" fn callback(_: i32, _: usize, _: i32, _: usize, _: usize, message: usize) {
			eprintln!("EGL ERROR: {:?}", unsafe {
				std::ffi::CStr::from_ptr(message as _)
			})
		}

		debug_message_control(&[0x3038], callback);
	}

	Ok(())
}

pub fn create_image(target: usize, attributes: &[i32]) -> usize {
	unsafe {
		let func: extern "C" fn(usize, usize, usize, usize, usize) -> usize =
			std::mem::transmute(EGL_CREATE_IMAGE_KHR);

		func(EGL_DISPLAY, 0, target, 0, attributes.as_ptr() as _)
	}
}

pub fn image_target_texture_2d_oes(target: i32, image: usize) {
	unsafe {
		let func: extern "C" fn(i32, usize) = std::mem::transmute(EGL_IMAGE_TARGET_TEXTURE_2D_OES);
		func(target, image)
	}
}

pub fn debug_message_control(
	target: &[i64],
	callback: extern "C" fn(i32, usize, i32, usize, usize, usize),
) -> i32 {
	unsafe {
		let func: extern "C" fn(usize, usize) -> i32 =
			std::mem::transmute(EGL_DEBUG_MESSAGE_CONTROL_KHR);

		func(callback as _, target.as_ptr() as _)
	}
}
