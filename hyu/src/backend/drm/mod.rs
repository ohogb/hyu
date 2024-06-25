use std::io::Read as _;

use crate::Result;

mod device;
mod egl;
mod gbm;

use device::*;
use glow::HasContext as _;

#[link(name = "EGL")]
extern "C" {
	fn eglGetProcAddress(name: *const i8) -> usize;
	fn eglBindAPI(api: u32) -> u32;
}

pub fn run() -> Result<()> {
	let device = Device::open("/dev/dri/card1")?;
	let resources = device.get_resources()?;

	let connectors = resources
		.connector_ids()
		.iter()
		.map(|x| device.get_connector(*x))
		.collect::<Result<Vec<_>>>()?;

	let connector = connectors.iter().find(|x| x.connection == 1).unwrap();

	let mode = connector
		.modes()
		.iter()
		.find(|x| (x.typee & (1 << 3)) != 0)
		.unwrap();

	let encoder = device.get_encoder(connector.encoder_id)?;

	let gbm_device = gbm::Device::create(device.get_fd());
	let gbm_surface = gbm_device
		.create_surface(
			mode.hdisplay as _,
			mode.vdisplay as _,
			0x34325258,
			(1 << 0) | (1 << 2),
		)
		.ok_or("failed to create gbm surface")?;

	let cstring = std::ffi::CString::new("eglGetPlatformDisplayEXT")?;
	let egl_get_platform_display = unsafe {
		std::mem::transmute::<
			_,
			extern "C" fn(
				platform: u32,
				native_display: u64,
				attrib_list: u64,
			) -> Option<egl::Display>,
		>(eglGetProcAddress(cstring.as_ptr()))
	};

	let display = egl_get_platform_display(0x31D7, gbm_device.as_ptr(), 0)
		.ok_or("failed to get platform display")?;

	crate::backend::gl::egl_wrapper::init(display.get_ptr() as _, |name| {
		let cstring = std::ffi::CString::new(name)?;
		Ok(unsafe { eglGetProcAddress(cstring.as_ptr()) })
	})?;

	display
		.initialize()
		.ok_or("failed to initialize egl display")?;

	if unsafe { eglBindAPI(0x30A0) } != 1 {
		Err("failed to bind gl api")?;
	}

	let configs = display.choose_config(
		&[
			0x3024, 8, 0x3023, 8, 0x3022, 8, 0x3021, 0, 0x3040, 0x0040, 0x3038,
		],
		100,
	);

	let config = configs
		.iter()
		.find(|config| {
			let ret = display.get_config_attrib(&config, 0x302E).unwrap();
			ret == 0x34325258
		})
		.ok_or("failed to find config with gbm format")?;

	let context = display
		.create_context(&config, &[0x3098, 3, 0x30FB, 2, 0x3038])
		.ok_or("failed to create context")?;

	let window_surface = display
		.create_window_surface(&config, gbm_surface.as_ptr(), &[0x3038])
		.ok_or("failed to create window surface")?;

	display.make_current(&window_surface, &context);

	let glow = unsafe {
		glow::Context::from_loader_function(|x| {
			let cstring = std::ffi::CString::new(x).unwrap();
			eglGetProcAddress(cstring.as_ptr()) as _
		})
	};

	unsafe {
		glow.clear_color(0.0, 0.0, 0.0, 1.0);
		glow.clear(glow::COLOR_BUFFER_BIT);
	}

	display.swap_buffers(&window_surface);

	let bo = gbm_surface
		.lock_front_buffer()
		.ok_or("failed to lock front buffer")?;

	let fb = bo.get_fb(&device)?;

	device.set_crtc(encoder.crtc_id, fb, 0, 0, &[connector.connector_id], &mode)?;

	let mut old_bo = bo;

	let mut fds = nix::sys::select::FdSet::new();
	fds.insert(unsafe { std::os::fd::BorrowedFd::borrow_raw(device.get_fd()) });

	let mut renderer = crate::backend::gl::Renderer::create(glow, 2560, 1440)?;

	loop {
		renderer.before()?;

		display.swap_buffers(&window_surface);

		let bo = gbm_surface
			.lock_front_buffer()
			.ok_or("failed to lock front buffer")?;

		let fb = bo.get_fb(&device)?;

		let mut has_flipped = false;
		device.page_flip(encoder.crtc_id, fb, 0x1, &mut has_flipped as *mut _ as _)?;

		while !has_flipped {
			nix::sys::select::select(device.get_fd() + 1, Some(&mut fds), None, None, None)?;

			let mut ret = [0u8; 0x1000];

			let amount = nix::unistd::read(device.get_fd(), &mut ret)?;
			assert!(amount == 32);

			let mut drm_event = ret.take(0x8);

			let mut typee = [0u8; 0x4];
			drm_event.read_exact(&mut typee)?;
			let typee = u32::from_ne_bytes(typee);

			let mut length = [0u8; 0x4];
			drm_event.read_exact(&mut length)?;
			let _length = u32::from_ne_bytes(length);

			match typee {
				2 => {
					let mut vblank = ret.take(24);

					let mut user_data = [0u8; 0x8];
					vblank.read_exact(&mut user_data)?;

					let mut user_data = [0u8; 0x8];
					vblank.read_exact(&mut user_data)?;
					let user_data = u64::from_ne_bytes(user_data);

					let user_data = user_data as *mut bool;

					unsafe {
						*user_data = true;
					}
				}
				_ => Err("unknown event")?,
			}
		}

		renderer.after()?;

		gbm_surface.release_buffer(old_bo);
		old_bo = bo;
	}
}
