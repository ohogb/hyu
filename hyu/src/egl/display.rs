use crate::{egl, gbm};

#[link(name = "EGL")]
unsafe extern "C" {
	fn eglInitialize(display: u64, major: &mut i32, minor: &mut i32) -> u32;
	fn eglChooseConfig(
		display: u64,
		attrib_list: u64,
		configs: u64,
		config_size: i32,
		num_config: u64,
	) -> u32;
	fn eglCreateContext(display: u64, config: u64, context: u64, attrib_list: u64) -> u64;
	fn eglGetConfigAttrib(display: u64, config: u64, attribute: i32, value: u64) -> u32;
	fn eglCreateWindowSurface(
		display: u64,
		config: u64,
		native_window: u64,
		attrib_list: u64,
	) -> u64;
	fn eglMakeCurrent(display: u64, draw: u64, read: u64, context: u64) -> u32;
	fn eglSwapBuffers(display: u64, surface: u64) -> u32;
	fn eglQuerySurface(display: u64, surface: u64, attribute: i32, value: u64) -> u32;
	fn eglGetProcAddress(name: *const i8) -> usize;
	fn eglDestroyImage(display: u64, image: u64) -> u32;
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Display {
	ptr: std::num::NonZeroU64,
}

impl Display {
	pub fn from_gbm(gbm_device: &gbm::Device) -> Option<Self> {
		static EGL_GET_PLATFORM_DISPLAY: std::sync::LazyLock<
			extern "C" fn(
				platform: u32,
				native_display: u64,
				attrib_list: u64,
			) -> Option<egl::Display>,
		> = std::sync::LazyLock::new(|| unsafe {
			std::mem::transmute(eglGetProcAddress(c"eglGetPlatformDisplayEXT".as_ptr()))
		});

		EGL_GET_PLATFORM_DISPLAY(0x31D7, gbm_device.as_ptr(), 0)
	}

	pub fn initialize(&self) -> Option<(i32, i32)> {
		let mut major = 0;
		let mut minor = 0;

		let ret = unsafe { eglInitialize(self.as_ptr(), &mut major, &mut minor) };

		if ret == 1 {
			Some((major, minor))
		} else {
			None
		}
	}

	pub fn choose_config(&self, attributes: &[i32], amount: i32) -> Vec<egl::Config> {
		let mut ret = (0..amount)
			.map(|_| std::mem::MaybeUninit::<egl::Config>::zeroed())
			.collect::<Vec<_>>();

		let mut num_configs = 0i32;

		let success = unsafe {
			eglChooseConfig(
				self.as_ptr(),
				attributes.as_ptr() as _,
				ret.as_mut_ptr() as _,
				amount,
				&mut num_configs as *const _ as _,
			)
		};

		if success == 1 {
			ret.into_iter()
				.take(num_configs as _)
				.map(|x| unsafe { x.assume_init() })
				.collect()
		} else {
			Vec::new()
		}
	}

	pub fn create_context(&self, config: &egl::Config, attributes: &[i32]) -> Option<egl::Context> {
		unsafe {
			std::mem::transmute(eglCreateContext(
				self.as_ptr(),
				config.as_ptr(),
				0,
				attributes.as_ptr() as _,
			))
		}
	}

	pub fn get_config_attrib(&self, config: &egl::Config, attribute: i32) -> Option<i32> {
		let mut ret = 0i32;

		let success = unsafe {
			eglGetConfigAttrib(
				self.as_ptr(),
				config.as_ptr(),
				attribute,
				&mut ret as *mut _ as _,
			)
		};

		if success == 1 {
			Some(ret)
		} else {
			None
		}
	}

	pub fn create_window_surface(
		&self,
		config: &egl::Config,
		native_window: u64,
		attributes: &[i32],
	) -> Option<egl::Surface> {
		unsafe {
			std::mem::transmute(eglCreateWindowSurface(
				self.as_ptr(),
				config.as_ptr(),
				native_window,
				attributes.as_ptr() as _,
			))
		}
	}

	pub fn make_current(
		&self,
		surface: Option<&egl::Surface>,
		context: Option<&egl::Context>,
	) -> crate::Result<()> {
		let ret = unsafe {
			eglMakeCurrent(
				self.as_ptr(),
				surface.map(|x| x.as_ptr()).unwrap_or(0),
				surface.map(|x| x.as_ptr()).unwrap_or(0),
				context.map(|x| x.as_ptr()).unwrap_or(0),
			)
		};

		if ret == 1 {
			Ok(())
		} else {
			color_eyre::eyre::bail!("make_current failed");
		}
	}

	pub fn swap_buffers(&self, surface: &egl::Surface) -> u32 {
		unsafe { eglSwapBuffers(self.as_ptr(), surface.as_ptr()) }
	}

	pub fn query_surface(&self, surface: &egl::Surface, attribute: i32) -> Option<i32> {
		let mut ret = 0i32;

		let success = unsafe {
			eglQuerySurface(
				self.as_ptr(),
				surface.as_ptr(),
				attribute,
				&mut ret as *mut _ as _,
			)
		};

		if success == 1 {
			Some(ret)
		} else {
			None
		}
	}

	pub fn create_image(&self, target: u64, attributes: &[i32]) -> Option<egl::Image> {
		static EGL_CREATE_IMAGE_KHR: std::sync::LazyLock<
			extern "C" fn(u64, u64, u64, u64, u64) -> Option<egl::Image>,
		> = std::sync::LazyLock::new(|| unsafe {
			std::mem::transmute(eglGetProcAddress(c"eglCreateImageKHR".as_ptr()))
		});

		EGL_CREATE_IMAGE_KHR(self.as_ptr(), 0, target, 0, attributes.as_ptr() as _)
	}

	pub fn destroy_image(&self, image: &egl::Image) -> bool {
		unsafe { eglDestroyImage(self.as_ptr(), image.as_ptr()) != 0 }
	}

	pub fn as_ptr(&self) -> u64 {
		self.ptr.get()
	}
}
