use crate::{egl, rt, state, Result};

mod device;
pub mod gbm;

use device::*;
use glow::HasContext as _;

pub struct State {
	device: Device,
	gbm_device: gbm::Device,
	egl_display: egl::Display,
	screen: Screen,
	renderer: crate::backend::gl::Renderer,
	context: AtomicHelper,
	pub render_tx: rt::producers::Sender<()>,
	render_rx: Option<rt::producers::Channel<()>>,
}

pub enum ScreenState {
	WaitingForPageFlip {
		bo: gbm::BufferObject,
		needs_rerender: bool,
	},
	Idle,
}

struct Screen {
	connector: PropWrapper<Connector>,
	mode: ModeInfo,
	encoder: Encoder,
	crtc: PropWrapper<Crtc>,
	plane: PropWrapper<Plane>,
	gbm_surface: gbm::Surface,

	connector_crtc_id: u32,

	crtc_mode_id: u32,
	crtc_active: u32,

	plane_fb_id: u32,
	plane_crtc_id: u32,
	plane_src_x: u32,
	plane_src_y: u32,
	plane_src_w: u32,
	plane_src_h: u32,
	plane_crtc_x: u32,
	plane_crtc_y: u32,
	plane_crtc_w: u32,
	plane_crtc_h: u32,

	window_surface: egl::Surface,
	old_bo: Option<gbm::BufferObject>,

	state: ScreenState,
}

impl Screen {
	pub fn create(
		connector: Connector,
		device: &Device,
		resources: &Card,
		gbm_device: &gbm::Device,
		display: &egl::Display,
		config: &egl::Config,
	) -> Result<Self> {
		let connector = PropWrapper::new(connector, &device);

		let mode = connector
			.modes()
			.iter()
			.find(|x| (x.typee & (1 << 3)) != 0)
			.unwrap()
			.clone();

		let encoder = device.get_encoder(connector.encoder_id)?;

		let crtc = PropWrapper::new(device.get_crtc(encoder.crtc_id)?, &device);

		let plane_resources = device.get_plane_resources()?;
		dbg!(&plane_resources);

		let planes = plane_resources
			.plane_ids()
			.iter()
			.map(|x| device.get_plane(*x))
			.collect::<Result<Vec<_>>>()?;

		let crtc_index = resources
			.crtc_ids()
			.iter()
			.enumerate()
			.find(|x| *x.1 == encoder.crtc_id)
			.map(|x| x.0)
			.unwrap();

		let plane = planes
			.iter()
			.find(|x| {
				if (x.possible_crtcs & (1 << crtc_index)) == 0 {
					return false;
				}

				let props = x.get_props(&device).unwrap();

				for (&id, &value) in std::iter::zip(props.prop_ids(), props.prop_values()) {
					let prop = device.get_prop(id).unwrap();

					if &prop.name[..4] == b"type" && value == 2 {
						return true;
					}
				}

				true
			})
			.unwrap();

		let plane = PropWrapper::new(plane.clone(), &device);

		let gbm_surface = gbm_device
			.create_surface(
				mode.hdisplay as _,
				mode.vdisplay as _,
				0x34325258,
				(1 << 0) | (1 << 2),
			)
			.ok_or("failed to create gbm surface")?;

		let connector_crtc_id = connector.find_property("CRTC_ID").unwrap();

		let crtc_mode_id = crtc.find_property("MODE_ID").unwrap();
		let crtc_active = crtc.find_property("ACTIVE").unwrap();

		let plane_fb_id = plane.find_property("FB_ID").unwrap();
		let plane_crtc_id = plane.find_property("CRTC_ID").unwrap();
		let plane_src_x = plane.find_property("SRC_X").unwrap();
		let plane_src_y = plane.find_property("SRC_Y").unwrap();
		let plane_src_w = plane.find_property("SRC_W").unwrap();
		let plane_src_h = plane.find_property("SRC_H").unwrap();
		let plane_crtc_x = plane.find_property("CRTC_X").unwrap();
		let plane_crtc_y = plane.find_property("CRTC_Y").unwrap();
		let plane_crtc_w = plane.find_property("CRTC_W").unwrap();
		let plane_crtc_h = plane.find_property("CRTC_H").unwrap();

		let window_surface = display
			.create_window_surface(config, gbm_surface.as_ptr(), &[0x3038])
			.ok_or("failed to create window surface")?;

		Ok(Self {
			connector,
			mode,
			encoder,
			crtc,
			plane,
			gbm_surface,
			connector_crtc_id,
			crtc_mode_id,
			crtc_active,
			plane_fb_id,
			plane_crtc_id,
			plane_src_x,
			plane_src_y,
			plane_src_w,
			plane_src_h,
			plane_crtc_x,
			plane_crtc_y,
			plane_crtc_w,
			plane_crtc_h,
			window_surface,
			old_bo: None,
			state: ScreenState::Idle,
		})
	}

	pub fn render(
		&mut self,
		device: &Device,
		display: &egl::Display,
		ctx: &mut AtomicHelper,
		modeset: bool,
	) -> Result<()> {
		let bo = self
			.gbm_surface
			.lock_front_buffer()
			.ok_or("failed to lock front buffer")?;

		let fb = bo.get_fb(&device)?;

		if modeset {
			ctx.add_property(
				&self.connector,
				self.connector_crtc_id,
				self.crtc.get_id() as _,
			);

			let blob = device.create_blob(unsafe {
				std::slice::from_raw_parts(
					&self.mode as *const _ as *const u8,
					std::mem::size_of::<ModeInfo>(),
				)
			})?;

			ctx.add_property(&self.crtc, self.crtc_mode_id, blob as _);
			ctx.add_property(&self.crtc, self.crtc_active, 1);
		}

		ctx.add_property(&self.plane, self.plane_fb_id, fb as _);
		ctx.add_property(&self.plane, self.plane_crtc_id, self.crtc.get_id() as _);
		ctx.add_property(&self.plane, self.plane_src_x, 0);
		ctx.add_property(&self.plane, self.plane_src_y, 0);
		ctx.add_property(
			&self.plane,
			self.plane_src_w,
			((self.mode.hdisplay as u32) << 16) as _,
		);
		ctx.add_property(
			&self.plane,
			self.plane_src_h,
			((self.mode.vdisplay as u32) << 16) as _,
		);

		ctx.add_property(&self.plane, self.plane_crtc_x, 0);
		ctx.add_property(&self.plane, self.plane_crtc_y, 0);
		ctx.add_property(&self.plane, self.plane_crtc_w, self.mode.hdisplay as _);
		ctx.add_property(&self.plane, self.plane_crtc_h, self.mode.vdisplay as _);

		let mut has_flipped = false;
		let mut flags = 0x200 | 0x1;

		if modeset {
			flags |= 0x400;
		}

		if false {
			flags |= 0x2;
		}

		device
			.commit(&ctx, flags, &mut has_flipped as *mut _ as _)
			.unwrap();

		ctx.clear();

		self.state = ScreenState::WaitingForPageFlip {
			bo,
			needs_rerender: false,
		};

		Ok(())
	}
}

pub fn initialize_state() -> Result<State> {
	let device = Device::open("/dev/dri/card1")?;
	device.set_client_capability(2, 1)?;
	device.set_client_capability(3, 1)?;

	let resources = device.get_resources()?;

	let connectors = resources
		.connector_ids()
		.iter()
		.map(|x| device.get_connector(*x))
		.collect::<Result<Vec<_>>>()?;

	let connectors = connectors
		.into_iter()
		.filter(|x| x.connection == 1)
		.collect::<Vec<_>>();

	let gbm_device = gbm::Device::create(device.get_fd());
	let display = egl::Display::from_gbm(&gbm_device).ok_or("failed to get platform display")?;

	egl::enable_debugging();

	display
		.initialize()
		.ok_or("failed to initialize egl display")?;

	if egl::bind_api(0x30A0) != 1 {
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
			let ret = display.get_config_attrib(config, 0x302E).unwrap();
			ret == 0x34325258
		})
		.ok_or("failed to find config with gbm format")?;

	let context = display
		.create_context(config, &[0x3098, 3, 0x30FB, 2, 0x3038])
		.ok_or("failed to create context")?;

	unsafe {
		crate::egl::DISPLAY.initialize(display.clone());
		crate::egl::CONTEXT.initialize(std::sync::Mutex::new(context.clone()));
	}

	let mut screen = Screen::create(
		connectors.iter().next().unwrap().clone(),
		&device,
		&resources,
		&gbm_device,
		&display,
		config,
	)?;

	let mut context_lock = crate::egl::CONTEXT.lock().unwrap();
	let access_holder = context_lock.access(&display, Some(&screen.window_surface))?;

	let glow = unsafe {
		glow::Context::from_loader_function(|x| {
			let cstring = std::ffi::CString::new(x).unwrap();
			egl::get_proc_address(&cstring) as _
		})
	};

	unsafe {
		glow.clear_color(0.0, 0.0, 0.0, 1.0);
		glow.clear(glow::COLOR_BUFFER_BIT);
	}

	display.swap_buffers(&screen.window_surface);

	let mut ctx = device.begin_atomic();
	screen.render(&device, &display, &mut ctx, true)?;

	let renderer = crate::backend::gl::Renderer::create(glow, 2560, 1440)?;

	drop(access_holder);
	drop(context_lock);

	let context = device.begin_atomic();

	let (tx, rx) = rt::producers::Channel::new()?;

	let state = State {
		device,
		gbm_device,
		egl_display: display,
		screen,
		renderer,
		context,
		render_tx: tx,
		render_rx: Some(rx),
	};

	Ok(state)
}

pub fn attach(runtime: &mut rt::Runtime<state::State>, state: &mut state::State) -> Result<()> {
	runtime.on(
		rt::producers::Drm::new(state.drm.device.get_fd()),
		|msg, state, _| {
			match msg {
				rt::producers::DrmMessage::PageFlip {
					tv_sec,
					tv_usec,
					sequence,
					..
				} => {
					let ScreenState::WaitingForPageFlip { bo, needs_rerender } =
						std::mem::replace(&mut state.drm.screen.state, ScreenState::Idle)
					else {
						panic!();
					};

					if let Some(old_bo) = std::mem::take(&mut state.drm.screen.old_bo) {
						state.drm.screen.gbm_surface.release_buffer(old_bo);
					}

					state.drm.screen.old_bo = Some(bo);

					state
						.drm
						.renderer
						.after(&mut state.compositor, tv_sec, tv_usec, sequence)?;

					if needs_rerender {
						state.drm.render_tx.send(())?;
					}
				}
			}

			Ok(())
		},
	);

	runtime.on(
		std::mem::take(&mut state.drm.render_rx).unwrap(),
		|_, state, _| {
			if let ScreenState::WaitingForPageFlip { needs_rerender, .. } =
				&mut state.drm.screen.state
			{
				*needs_rerender = true;
				return Ok(());
			}

			let mut context_lock = crate::egl::CONTEXT.lock().unwrap();
			let access_holder = context_lock.access(
				&state.drm.egl_display,
				Some(&state.drm.screen.window_surface),
			)?;

			state.drm.renderer.before(&mut state.compositor)?;
			state
				.drm
				.egl_display
				.swap_buffers(&state.drm.screen.window_surface);

			drop(access_holder);
			drop(context_lock);

			state.drm.screen.render(
				&state.drm.device,
				&state.drm.egl_display,
				&mut state.drm.context,
				false,
			)
		},
	);

	// initial render
	state.drm.render_tx.send(())?;

	Ok(())
}
