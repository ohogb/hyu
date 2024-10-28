mod config;
mod context;
mod display;
mod image;
mod surface;

pub use config::*;
pub use context::*;
pub use display::*;
pub use image::*;
pub use surface::*;

#[link(name = "EGL")]
unsafe extern "C" {
	fn eglGetProcAddress(name: *const i8) -> usize;
	fn eglBindAPI(api: u32) -> u32;
}

pub fn get_proc_address(str: &std::ffi::CStr) -> usize {
	unsafe { eglGetProcAddress(str.as_ptr()) }
}

pub fn bind_api(api: u32) -> u32 {
	unsafe { eglBindAPI(api) }
}

pub fn enable_debugging() {
	static EGL_DEBUG_MESSAGE_CONTROL_KHR: std::sync::LazyLock<extern "C" fn(u64, u64) -> i32> =
		std::sync::LazyLock::new(|| unsafe {
			std::mem::transmute(eglGetProcAddress(c"eglDebugMessageControlKHR".as_ptr()))
		});

	extern "C" fn callback(err: i32, _: usize, _: i32, _: usize, _: usize, message: usize) {
		eprintln!(
			"EGL ERROR: {:?} {:X}",
			unsafe { std::ffi::CStr::from_ptr(message as _) },
			err
		)
	}

	EGL_DEBUG_MESSAGE_CONTROL_KHR(callback as _, [0x3038].as_ptr() as _);
}

// TODO: get rid of this
pub static DISPLAY: crate::GlobalWrapper<Display> = crate::GlobalWrapper::empty();
