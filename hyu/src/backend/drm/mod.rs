use crate::{
	Result,
	drm::{self, HasProps as _, Object as _},
	elp, gbm, state,
};

use color_eyre::eyre::OptionExt as _;

pub struct State {
	pub device: drm::Device,
	#[expect(dead_code)]
	gbm_device: gbm::Device,
	pub screen: Screen,
	context: drm::AtomicHelper,
	pub vulkan: crate::renderer::vulkan::Renderer,
}

pub enum ScreenState {
	WaitingForPageFlip {},
	Idle,
}

pub struct Screen {
	connector: drm::PropWrapper<drm::Connector>,
	pub mode: drm::ModeInfo,
	#[expect(dead_code)]
	encoder: drm::Encoder,
	crtc: drm::PropWrapper<drm::Crtc>,
	plane: drm::PropWrapper<drm::Plane>,
	props: Props,

	buffers: [(
		gbm::BufferObject,
		ash::vk::Image,
		ash::vk::ImageView,
		ash::vk::Framebuffer,
	); 2],

	state: ScreenState,

	timer_tx: std::sync::Arc<nix::sys::timerfd::TimerFd>,
	timer_rx: Option<elp::timer_fd::Source>,
}

struct ConnectorProps {
	crtc_id: u32,
}

struct CrtcProps {
	mode_id: u32,
	active: u32,
}

struct PlaneProps {
	fb_id: u32,
	crtc_id: u32,
	src_x: u32,
	src_y: u32,
	src_w: u32,
	src_h: u32,
	crtc_x: u32,
	crtc_y: u32,
	crtc_w: u32,
	crtc_h: u32,
}

struct Props {
	connector: ConnectorProps,
	crtc: CrtcProps,
	plane: PlaneProps,
}

impl Screen {
	pub fn create(
		connector: drm::Connector,
		device: &drm::Device,
		resources: &drm::Card,
		gbm_device: &gbm::Device,
		vulkan: &crate::renderer::vulkan::Renderer,
	) -> Result<Self> {
		let connector = drm::PropWrapper::new(connector, device);

		let mode = connector
			.modes()
			.iter()
			.find(|x| (x.typee & (1 << 3)) != 0)
			.unwrap()
			.clone();

		let encoder = device.get_encoder(connector.encoder_id)?;

		let crtc = drm::PropWrapper::new(device.get_crtc(encoder.crtc_id)?, device);

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

				let props = x.get_props(device).unwrap();

				for (&id, &value) in std::iter::zip(props.prop_ids(), props.prop_values()) {
					let prop = device.get_prop(id).unwrap();

					if &prop.name[..4] == b"type" && value == 2 {
						return true;
					}
				}

				true
			})
			.unwrap();

		let plane = drm::PropWrapper::new(plane.clone(), device);

		let asdf = <[_; 2]>::try_from(
			(0..2)
				.map(|_| {
					let bo = gbm_device
						.create_buffer_object(
							mode.hdisplay as _,
							mode.vdisplay as _,
							0x34325258,
							&[0],
							(1 << 0) | (1 << 2),
						)
						.ok_or_eyre("failed to create buffer object")?;

					let (image, image_view) = vulkan.create_image_from_gbm(&bo)?;

					let attachments = [image_view];

					let framebuffer_create_info = ash::vk::FramebufferCreateInfo::default()
						.render_pass(vulkan.render_pass)
						.attachments(&attachments)
						.width(2560)
						.height(1440)
						.layers(1);

					let framebuffer = unsafe {
						vulkan
							.device
							.create_framebuffer(&framebuffer_create_info, None)?
					};

					Ok((bo, image, image_view, framebuffer))
				})
				.into_iter()
				.collect::<Result<Vec<_>>>()?,
		)
		.unwrap();

		let props = Props {
			connector: ConnectorProps {
				crtc_id: connector.find_property("CRTC_ID").unwrap(),
			},
			crtc: CrtcProps {
				mode_id: crtc.find_property("MODE_ID").unwrap(),
				active: crtc.find_property("ACTIVE").unwrap(),
			},
			plane: PlaneProps {
				fb_id: plane.find_property("FB_ID").unwrap(),
				crtc_id: plane.find_property("CRTC_ID").unwrap(),
				src_x: plane.find_property("SRC_X").unwrap(),
				src_y: plane.find_property("SRC_Y").unwrap(),
				src_w: plane.find_property("SRC_W").unwrap(),
				src_h: plane.find_property("SRC_H").unwrap(),
				crtc_x: plane.find_property("CRTC_X").unwrap(),
				crtc_y: plane.find_property("CRTC_Y").unwrap(),
				crtc_w: plane.find_property("CRTC_W").unwrap(),
				crtc_h: plane.find_property("CRTC_H").unwrap(),
			},
		};

		let (timer_tx, timer_rx) = elp::timer_fd::create()?;

		Ok(Self {
			connector,
			mode,
			encoder,
			crtc,
			plane,
			props,
			state: ScreenState::Idle,
			timer_tx,
			timer_rx: Some(timer_rx),
			buffers: asdf,
		})
	}

	pub fn render(
		&mut self,
		device: &drm::Device,
		ctx: &mut drm::AtomicHelper,
		modeset: bool,
	) -> Result<()> {
		let (bo, ..) = self.buffers.first().unwrap();
		let fb = bo.get_fb(device)?;

		if modeset {
			ctx.add_property(
				&self.connector,
				self.props.connector.crtc_id,
				self.crtc.get_id() as _,
			);

			let blob = device.create_blob(unsafe {
				std::slice::from_raw_parts(
					&self.mode as *const _ as *const u8,
					std::mem::size_of::<drm::ModeInfo>(),
				)
			})?;

			ctx.add_property(&self.crtc, self.props.crtc.mode_id, blob as _);
			ctx.add_property(&self.crtc, self.props.crtc.active, 1);
		}

		ctx.add_property(&self.plane, self.props.plane.fb_id, fb as _);
		ctx.add_property(
			&self.plane,
			self.props.plane.crtc_id,
			self.crtc.get_id() as _,
		);
		ctx.add_property(&self.plane, self.props.plane.src_x, 0);
		ctx.add_property(&self.plane, self.props.plane.src_y, 0);
		ctx.add_property(
			&self.plane,
			self.props.plane.src_w,
			((self.mode.hdisplay as u32) << 16) as _,
		);
		ctx.add_property(
			&self.plane,
			self.props.plane.src_h,
			((self.mode.vdisplay as u32) << 16) as _,
		);

		ctx.add_property(&self.plane, self.props.plane.crtc_x, 0);
		ctx.add_property(&self.plane, self.props.plane.crtc_y, 0);
		ctx.add_property(
			&self.plane,
			self.props.plane.crtc_w,
			self.mode.hdisplay as _,
		);
		ctx.add_property(
			&self.plane,
			self.props.plane.crtc_h,
			self.mode.vdisplay as _,
		);

		let mut has_flipped = false;
		let mut flags = 0x200 | 0x1;

		if modeset {
			flags |= 0x400;
		} else if false {
			flags |= 0x2;
		}

		device
			.commit(ctx, flags, &mut has_flipped as *mut _ as _)
			.unwrap();

		ctx.clear();

		self.state = ScreenState::WaitingForPageFlip {};

		Ok(())
	}
}

pub fn initialize_state(card: impl AsRef<std::path::Path>) -> Result<State> {
	let device = drm::Device::open(&card)?;
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

	let mut vk = crate::renderer::vulkan::create(card)?;
	eprintln!("VK: {:#?} {:#?}", vk.physical_device, vk.queue);

	let mut screen = Screen::create(
		connectors.first().unwrap().clone(),
		&device,
		&resources,
		&gbm_device,
		&vk,
	)?;

	let &(_, image, _, framebuffer) = screen.buffers.first().unwrap();
	vk.render(image, framebuffer, |_| Ok(()))?;

	let mut ctx = device.begin_atomic();
	screen.render(&device, &mut ctx, true)?;

	let context = device.begin_atomic();

	let state = State {
		device,
		gbm_device,
		screen,
		context,
		vulkan: vk,
	};

	Ok(state)
}

pub fn attach(
	event_loop: &mut elp::EventLoop<state::State>,
	state: &mut state::State,
) -> Result<()> {
	event_loop.on(
		elp::drm::create(state.hw.drm.device.get_fd()),
		|msg, state, _| {
			match msg {
				elp::drm::Message::PageFlip {
					tv_sec,
					tv_usec,
					sequence,
					..
				} => {
					let ScreenState::WaitingForPageFlip {} =
						std::mem::replace(&mut state.hw.drm.screen.state, ScreenState::Idle)
					else {
						panic!();
					};

					state.hw.drm.screen.buffers.swap(0, 1);

					let duration = std::time::Duration::from_micros(
						tv_sec as u64 * 1_000_000 + tv_usec as u64,
					);

					let till_next_refresh = std::time::Duration::from_micros(
						1_000_000 / state.hw.drm.screen.mode.vrefresh as u64,
					);

					state.compositor.after_render(
						duration,
						till_next_refresh,
						sequence,
						0x1 | 0x2 | 0x4,
					)?;

					let next_render =
						duration + till_next_refresh - std::time::Duration::from_micros(1_000);

					state.hw.drm.screen.timer_tx.set(
						nix::sys::timerfd::Expiration::OneShot(
							nix::sys::time::TimeSpec::from_duration(next_render),
						),
						nix::sys::timerfd::TimerSetTimeFlags::TFD_TIMER_ABSTIME,
					)?;
				}
			}

			Ok(())
		},
	)?;

	event_loop.on(
		std::mem::take(&mut state.hw.drm.screen.timer_rx).unwrap(),
		|_, state, _| {
			if let ScreenState::WaitingForPageFlip { .. } = &state.hw.drm.screen.state {
				panic!();
			}

			let &(_, image, _, framebuffer) = state.hw.drm.screen.buffers.first().unwrap();
			state
				.hw
				.drm
				.vulkan
				.render(image, framebuffer, |vk| state.compositor.render(vk))?;

			state
				.hw
				.drm
				.screen
				.render(&state.hw.drm.device, &mut state.hw.drm.context, false)
		},
	)
}
