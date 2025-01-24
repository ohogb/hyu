use crate::{
	Point, Result,
	drm::{self, HasProps as _, Object as _},
	elp, gbm, state, wl,
};

use color_eyre::eyre::OptionExt as _;

pub struct State {
	pub device: drm::Device,
	gbm_device: std::mem::ManuallyDrop<gbm::Device>,
	pub screen: Screen,
	context: drm::AtomicHelper,
	pub vulkan: crate::renderer::vulkan::Renderer,
}

pub enum ScreenState {
	WaitingForPageFlip { did_direct_scanout: bool },
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
		std::mem::ManuallyDrop<gbm::BufferObject>,
		ash::vk::Image,
		ash::vk::ImageView,
		ash::vk::Framebuffer,
		ash::vk::CommandBuffer,
	); 2],

	state: ScreenState,

	pub timer_tx: std::sync::Arc<nix::sys::timerfd::TimerFd>,
	timer_rx: Option<elp::timer_fd::Source>,

	last_refresh: Option<std::time::Duration>,
}

struct ConnectorProps {
	crtc_id: u32,
}

struct CrtcProps {
	mode_id: u32,
	active: u32,
	#[expect(dead_code)]
	vrr_enabled: u32,
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
	in_fence_fd: u32,
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

					if &prop.name[..4] == b"type" && value == 1 {
						return true;
					}
				}

				false
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

					let command_pool_create_info = ash::vk::CommandPoolCreateInfo::default()
						.flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

					let command_pool = unsafe {
						vulkan
							.device
							.create_command_pool(&command_pool_create_info, None)?
					};

					let command_buffer_allocation_info =
						ash::vk::CommandBufferAllocateInfo::default()
							.command_pool(command_pool)
							.level(ash::vk::CommandBufferLevel::PRIMARY)
							.command_buffer_count(1);

					let command_buffers = unsafe {
						vulkan
							.device
							.allocate_command_buffers(&command_buffer_allocation_info)?
					};

					let command_buffer = command_buffers.into_iter().next().unwrap();

					Ok((
						std::mem::ManuallyDrop::new(bo),
						image,
						image_view,
						framebuffer,
						command_buffer,
					))
				})
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
				vrr_enabled: crtc.find_property("VRR_ENABLED").unwrap(),
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
				in_fence_fd: plane.find_property("IN_FENCE_FD").unwrap(),
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
			last_refresh: None,
		})
	}

	pub fn render(
		&self,
		device: &drm::Device,
		ctx: &mut drm::AtomicHelper,
		modeset: bool,
		in_fence_fd: std::os::fd::RawFd,
		bo: &gbm::BufferObject,
	) -> Result<()> {
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
			// ctx.add_property(&self.crtc, self.props.crtc.vrr_enabled, 1);
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
		ctx.add_property(&self.plane, self.props.plane.in_fence_fd, in_fence_fd as _);

		let mut flags = 0x200 | 0x1;

		if modeset {
			flags |= 0x400;
		} else if false {
			flags |= 0x2;
		}

		device.commit(ctx, flags, std::ptr::null_mut())?;
		ctx.clear();

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

	let gbm_device =
		gbm::Device::create(device.get_fd()).ok_or_eyre("failed to create gbm device")?;

	let mut vk = crate::renderer::vulkan::create(card)?;
	eprintln!("VK: {:#?} {:#?}", vk.physical_device, vk.queue);

	let mut screen = Screen::create(
		connectors.first().unwrap().clone(),
		&device,
		&resources,
		&gbm_device,
		&vk,
	)?;

	let &(_, image, _, framebuffer, command_buffer) = screen.buffers.first().unwrap();
	vk.render(image, framebuffer, command_buffer, |_| Ok(()))?;

	let mut ctx = device.begin_atomic();
	screen.render(
		&device,
		&mut ctx,
		true,
		-1,
		&screen.buffers.first().unwrap().0,
	)?;

	screen.state = ScreenState::WaitingForPageFlip {
		did_direct_scanout: false,
	};

	let context = device.begin_atomic();

	let state = State {
		device,
		gbm_device: std::mem::ManuallyDrop::new(gbm_device),
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
					let ScreenState::WaitingForPageFlip { did_direct_scanout } =
						std::mem::replace(&mut state.hw.drm.screen.state, ScreenState::Idle)
					else {
						panic!();
					};

					state.hw.drm.screen.buffers.swap(0, 1);

					let refresh_time = std::time::Duration::from_micros(
						tv_sec as u64 * 1_000_000 + tv_usec as u64,
					);

					let one_display_refresh_cycle = std::time::Duration::from_micros(
						1_000_000 / state.hw.drm.screen.mode.vrefresh as u64,
					);

					if let Some(last_refresh) = state.hw.drm.screen.last_refresh {
						let time_since_last_refresh = refresh_time.saturating_sub(last_refresh);

						let diff_from_expected_refresh_time =
							time_since_last_refresh.saturating_sub(one_display_refresh_cycle);

						if diff_from_expected_refresh_time > std::time::Duration::from_micros(500) {
							eprintln!("missed frame by {diff_from_expected_refresh_time:?}");
						}
					}

					state.hw.drm.screen.last_refresh = Some(refresh_time);

					let mut wp_presentation_flags = 0x1 | 0x2 | 0x4;

					if did_direct_scanout {
						wp_presentation_flags |= 0x8;
					}

					state.compositor.after_render(
						refresh_time,
						one_display_refresh_cycle,
						sequence,
						wp_presentation_flags,
					)?;

					let next_render = refresh_time + one_display_refresh_cycle
						- std::time::Duration::from_micros(1_000);

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
			let screen = &mut state.hw.drm.screen;

			if let ScreenState::WaitingForPageFlip { .. } = &screen.state {
				panic!();
			}

			if state.compositor.windows.len() == 1 {
				let window = state.compositor.windows.first().unwrap();
				let client = state.compositor.clients.get_mut(&window.0).unwrap();

				let xdg_toplevel = client.get_object(window.1)?;
				let xdg_surface = client.get_object(xdg_toplevel.surface)?;
				let wl_surface = client.get_object_mut(xdg_surface.surface)?;

				if wl_surface.children.len() == 0 {
					if let wl::SurfaceRenderTexture::AttachedDmabuf(attached_buffer) =
						&wl_surface.render_texture
					{
						let wl_buffer = client.get_object_mut(attached_buffer.wl_buffer_id)?;
						let wl::BufferBackingStorage::Dmabuf(dmabuf_backing_storage) =
							&mut wl_buffer.backing_storage
						else {
							panic!();
						};

						if dmabuf_backing_storage.size == Point(2560, 1440) {
							if dmabuf_backing_storage.gbm_buffer_object.is_none() {
								dmabuf_backing_storage.gbm_buffer_object = Some(
									state
										.hw
										.drm
										.gbm_device
										.import_dmabuf(&dmabuf_backing_storage.attributes)
										.ok_or_eyre("failed to import dmabuf as bo")?,
								);
							}

							let Some(gbm_buffer_object) = &dmabuf_backing_storage.gbm_buffer_object
							else {
								panic!();
							};

							screen.render(
								&state.hw.drm.device,
								&mut state.hw.drm.context,
								false,
								-1,
								&gbm_buffer_object,
							)?;

							if let Some(currently_renderer_buffer) = std::mem::replace(
								&mut wl_surface.currently_rendered_buffer,
								Some(attached_buffer.clone()),
							) {
								currently_renderer_buffer.release(client)?;
							}

							screen.state = ScreenState::WaitingForPageFlip {
								did_direct_scanout: true,
							};

							return Ok(());
						}
					}
				}
			}

			let (bo, image, _, framebuffer, command_buffer) = screen.buffers.first().unwrap();

			state
				.hw
				.drm
				.vulkan
				.render(*image, *framebuffer, *command_buffer, |vulkan| {
					state.compositor.render(vulkan)
				})?;

			screen.render(
				&state.hw.drm.device,
				&mut state.hw.drm.context,
				false,
				state.hw.drm.vulkan.semaphore_fd.unwrap(),
				bo,
			)?;

			screen.state = ScreenState::WaitingForPageFlip {
				did_direct_scanout: false,
			};

			Ok(())
		},
	)
}
