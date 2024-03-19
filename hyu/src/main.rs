#![feature(unix_socket_ancillary_data)]
#![feature(generic_arg_infer)]

mod state;
mod vertex;
pub mod wl;

pub use vertex::*;

pub use state::{Buffer, State};
use winit::platform::{
	scancode::PhysicalKeyExtScancode,
	wayland::{EventLoopBuilderExtWayland, WindowBuilderExtWayland},
};
use wl::Object;

use std::{
	io::{Read, Write},
	os::fd::AsRawFd,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

async fn render() -> Result<()> {
	env_logger::init();

	let event_loop = winit::event_loop::EventLoopBuilder::new()
		.with_any_thread(true)
		.build()?;

	let window = winit::window::WindowBuilder::new()
		.with_name("hyu", "hyu")
		.with_inner_size(winit::dpi::PhysicalSize::new(WIDTH as u32, HEIGHT as u32))
		.with_fullscreen(None)
		.build(&event_loop)?;

	let instance = wgpu::Instance::default();
	let surface = unsafe { instance.create_surface(&window)? };

	let adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions {
			compatible_surface: Some(&surface),
			..Default::default()
		})
		.await
		.unwrap();

	let (device, queue) = adapter
		.request_device(
			&wgpu::DeviceDescriptor {
				label: None,
				features: wgpu::Features::empty(),
				limits: wgpu::Limits::downlevel_defaults(),
			},
			None,
		)
		.await?;

	let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: None,
		source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl"))),
	});

	let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			},
		],
		label: None,
	});

	let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: None,
		bind_group_layouts: &[&bind_group_layout],
		push_constant_ranges: &[],
	});

	let caps = surface.get_capabilities(&adapter);

	surface.configure(
		&device,
		&wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: caps.formats[0],
			width: window.inner_size().width,
			height: window.inner_size().height,
			present_mode: wgpu::PresentMode::AutoVsync,
			alpha_mode: caps.alpha_modes[0],
			view_formats: Vec::new(),
		},
	);

	let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		label: None,
		size: (std::mem::size_of::<Vertex>() * WIDTH * HEIGHT * 8) as u64,
		usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
		mapped_at_creation: false,
	});

	let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

	let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: None,
		layout: Some(&pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: &[wgpu::VertexBufferLayout {
				array_stride: std::mem::size_of::<Vertex>() as _,
				step_mode: wgpu::VertexStepMode::Vertex,
				attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
			}],
		},
		fragment: Some(wgpu::FragmentState {
			module: &shader,
			entry_point: "fs_main",
			targets: &[Some(wgpu::ColorTargetState {
				format: caps.formats[0],
				blend: Some(wgpu::BlendState {
					color: wgpu::BlendComponent {
						src_factor: wgpu::BlendFactor::SrcAlpha,
						dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
						operation: wgpu::BlendOperation::Add,
					},
					alpha: wgpu::BlendComponent::OVER,
				}),
				write_mask: wgpu::ColorWrites::ALL,
			})],
		}),
		primitive: wgpu::PrimitiveState::default(),
		depth_stencil: None,
		multisample: wgpu::MultisampleState::default(),
		multiview: None,
	});

	let mut vertices = Vec::with_capacity(WIDTH * HEIGHT * 8);

	let start_time = std::time::Instant::now();

	event_loop.run(move |event, target| {
		let winit::event::Event::WindowEvent { window_id, event } = event else {
			return;
		};

		if window_id != window.id() {
			return;
		}

		match event {
			winit::event::WindowEvent::RedrawRequested => {
				let frame = surface.get_current_texture().unwrap();

				let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
					..Default::default()
				});

				let mut encoder =
					device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

				encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
					label: None,
					color_attachments: &[Some(wgpu::RenderPassColorAttachment {
						view: &view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(wgpu::Color {
								r: 0.2,
								g: 0.2,
								b: 0.2,
								a: 1.0,
							}),
							store: wgpu::StoreOp::Store,
						},
					})],
					depth_stencil_attachment: None,
					timestamp_writes: None,
					occlusion_query_set: None,
				});

				for client in state::clients().values_mut() {
					for window in client.windows.clone() {
						let window = client.get_object::<wl::XdgToplevel>(window).unwrap();

						let xdg_surface =
							client.get_object::<wl::XdgSurface>(window.surface).unwrap();

						let surface = client
							.get_object_mut::<wl::Surface>(xdg_surface.get_surface())
							.unwrap();

						surface
							.wgpu_do_textures(client, &device, &queue, &sampler, &bind_group_layout)
							.unwrap();

						surface
							.frame(start_time.elapsed().as_millis() as u32, client)
							.unwrap();

						for (x, y, width, height, surface_id) in surface.get_front_buffers(client) {
							let surface = client.get_object::<wl::Surface>(surface_id).unwrap();

							let Some((.., (_, bind_group))) = &surface.data else {
								panic!();
							};

							fn pixels_to_float(input: [i32; 2]) -> [f32; 2] {
								[
									input[0] as f32 / WIDTH as f32 * 2.0 - 1.0,
									(input[1] as f32 / HEIGHT as f32 * 2.0 - 1.0) * -1.0,
								]
							}

							let x = window.position.0 - xdg_surface.position.0 + x;
							let y = window.position.1 - xdg_surface.position.1 + y;

							vertices.push(Vertex {
								position: pixels_to_float([x, y]),
								uv: [0.0, 0.0],
							});

							vertices.push(Vertex {
								position: pixels_to_float([x + width, y]),
								uv: [1.0, 0.0],
							});

							vertices.push(Vertex {
								position: pixels_to_float([x, y + height]),
								uv: [0.0, 1.0],
							});

							vertices.push(Vertex {
								position: pixels_to_float([x, y + height]),
								uv: [0.0, 1.0],
							});

							vertices.push(Vertex {
								position: pixels_to_float([x + width, y + height]),
								uv: [1.0, 1.0],
							});

							vertices.push(Vertex {
								position: pixels_to_float([x + width, y]),
								uv: [1.0, 0.0],
							});

							let mut render_pass =
								encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
									label: None,
									color_attachments: &[Some(wgpu::RenderPassColorAttachment {
										view: &view,
										resolve_target: None,
										ops: wgpu::Operations {
											load: wgpu::LoadOp::Load,
											store: wgpu::StoreOp::Store,
										},
									})],
									depth_stencil_attachment: None,
									timestamp_writes: None,
									occlusion_query_set: None,
								});

							render_pass.set_pipeline(&render_pipeline);
							render_pass.set_bind_group(0, bind_group, &[]);
							render_pass.set_vertex_buffer(
								0,
								vertex_buffer.slice(
									((vertices.len() - 6) * std::mem::size_of::<Vertex>()) as u64..,
								),
							);
							render_pass.draw(0..6 as _, 0..1);
						}
					}
				}

				queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));

				queue.submit(Some(encoder.finish()));
				frame.present();

				vertices.clear();

				window.request_redraw();
			}
			winit::event::WindowEvent::CloseRequested => {
				target.exit();
			}
			winit::event::WindowEvent::CursorMoved {
				position: cursor_position,
				..
			} => {
				for client in state::clients().values_mut() {
					let old = client.surface_cursor_is_over;
					client.surface_cursor_is_over = None;

					fn do_stuff(
						client: &mut wl::Client,
						surface: &wl::Surface,
						cursor_position: (i32, i32),
						surface_position: (i32, i32),
						surface_size: (i32, i32),
					) {
						fn is_point_inside_area(
							cursor: (i32, i32),
							position: (i32, i32),
							size: (i32, i32),
						) -> bool {
							cursor.0 > position.0
								&& cursor.1 > position.1 && cursor.0 <= position.0 + size.0
								&& cursor.1 <= position.1 + size.1
						}

						if is_point_inside_area(cursor_position, surface_position, surface_size) {
							client.surface_cursor_is_over = Some((
								surface.object_id,
								(
									cursor_position.0 - surface_position.0,
									cursor_position.1 - surface_position.1,
								),
							));
						}

						for child in &surface.children {
							let sub_surface = client.get_object::<wl::SubSurface>(*child).unwrap();
							let surface = client
								.get_object::<wl::Surface>(sub_surface.surface)
								.unwrap();

							let size = if let Some((w, h, ..)) = surface.data {
								(w, h)
							} else {
								(0, 0)
							};

							do_stuff(
								client,
								surface,
								cursor_position,
								(
									surface_position.0 + sub_surface.position.0,
									surface_position.1 + sub_surface.position.1,
								),
								size,
							);
						}
					}

					for window in client.windows.clone() {
						let toplevel = client.get_object::<wl::XdgToplevel>(window).unwrap();
						let xdg_surface = client
							.get_object::<wl::XdgSurface>(toplevel.surface)
							.unwrap();

						let surface = client
							.get_object::<wl::Surface>(xdg_surface.get_surface())
							.unwrap();

						let position = (
							toplevel.position.0 - xdg_surface.position.0,
							toplevel.position.1 - xdg_surface.position.1,
						);

						// let size = xdg_surface.size;

						let Some((w, h, ..)) = surface.data else {
							continue;
						};

						do_stuff(client, surface, cursor_position.into(), position, (w, h));
					}

					for object in client.objects() {
						let wl::Resource::Pointer(pointer) = object else {
							continue;
						};

						if old.map(|x| x.0) != client.surface_cursor_is_over.map(|x| x.0) {
							if let Some((old, ..)) = old {
								pointer.leave(client, old).unwrap();
								println!("leave");
							}

							if let Some((surface, (x, y))) = client.surface_cursor_is_over {
								pointer.enter(client, surface, x, y).unwrap();
								println!("enter");
							}
						} else {
							if let Some((_, (x, y))) = client.surface_cursor_is_over {
								pointer.motion(client, x, y).unwrap();
							}
						}
					}
				}
			}
			winit::event::WindowEvent::MouseInput { state, button, .. } => match button {
				winit::event::MouseButton::Left => {
					let state = match state {
						winit::event::ElementState::Pressed => 1,
						winit::event::ElementState::Released => 0,
					};

					for client in state::clients().values_mut() {
						for object in client.objects() {
							let wl::Resource::Pointer(pointer) = object else {
								continue;
							};

							pointer.button(client, 0x110, state).unwrap();
						}
					}
				}
				_ => {}
			},
			winit::event::WindowEvent::KeyboardInput { event, .. } => {
				let code = event.physical_key.to_scancode().unwrap();

				let state = match event.state {
					winit::event::ElementState::Pressed => 1,
					winit::event::ElementState::Released => 0,
				};

				for client in state::clients().values_mut() {
					for object in client.objects() {
						let wl::Resource::Keyboard(keyboard) = object else {
							continue;
						};

						if !client.has_keyboard_focus {
							// keyboard.keymap(client).unwrap();

							for window in client.windows.clone() {
								let toplevel =
									client.get_object::<wl::XdgToplevel>(window).unwrap();

								let xdg_surface = client
									.get_object::<wl::XdgSurface>(toplevel.surface)
									.unwrap();

								keyboard.enter(client, xdg_surface.get_surface()).unwrap();
							}

							client.has_keyboard_focus = true;
						}

						if keyboard.key_states[code as usize] != (state != 0) {
							keyboard.key_states[code as usize] = state != 0;
							keyboard.key(client, code, state).unwrap();
						}
					}
				}
			}
			_ => {}
		}
	})?;

	Ok(())
}

fn client_event_loop(mut stream: std::os::unix::net::UnixStream, index: usize) -> Result<()> {
	stream.set_nonblocking(true)?;

	let mut client = wl::Client::new(State {
		buffer: Buffer(Vec::new()),
		start_position: ((100 * index + 10) as i32, (100 * index + 10) as i32),
	});

	let mut display = wl::Display::new(1);

	display.push_global(wl::Shm::new());
	display.push_global(wl::Compositor::new());
	display.push_global(wl::SubCompositor::new());
	display.push_global(wl::DataDeviceManager::new());
	display.push_global(wl::Seat::new());
	display.push_global(wl::Output::new());
	display.push_global(wl::XdgWmBase::new());

	client.queue_new_object(1, display);

	state::clients().insert(stream.as_raw_fd(), client);

	loop {
		{
			let mut clients = state::clients();
			let client = clients.get_mut(&stream.as_raw_fd()).unwrap();

			client.process_queue()?;

			let ret = stream.write_all(&client.get_state().buffer.0);

			if let Err(e) = ret {
				match e.kind() {
					std::io::ErrorKind::BrokenPipe => {
						clients.remove(&stream.as_raw_fd());
						return Ok(());
					}
					_ => {
						Err(e)?;
					}
				}
			}

			client.get_state().buffer.0.clear();
		}

		let mut cmsg_buffer = [0u8; 0x20];
		let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

		let mut obj = [0u8; 4];

		let len = stream
			.recv_vectored_with_ancillary(&mut [std::io::IoSliceMut::new(&mut obj)], &mut cmsg);

		let len = match len {
			Ok(len) => len,
			Err(x) => match x.kind() {
				std::io::ErrorKind::WouldBlock => {
					std::thread::sleep(std::time::Duration::from_millis(10));
					continue;
				}
				_ => {
					return Err(x)?;
				}
			},
		};

		let mut clients = state::clients();

		if len == 0 {
			clients.remove(&stream.as_raw_fd());
			return Ok(());
		}

		let client = clients.get_mut(&stream.as_raw_fd()).unwrap();

		for i in cmsg.messages() {
			let std::os::unix::net::AncillaryData::ScmRights(scm_rights) = i.unwrap() else {
				continue;
			};

			client.push_fds(scm_rights.into_iter().collect());
		}

		let mut op = [0u8; 2];
		stream.read_exact(&mut op).unwrap();

		let mut size = [0u8; 2];
		stream.read_exact(&mut size).unwrap();

		let size = u16::from_ne_bytes(size) - 0x8;

		let mut params = Vec::new();
		let _ = (&mut stream)
			.take(size as _)
			.read_to_end(&mut params)
			.unwrap();

		let object = u32::from_ne_bytes(obj);
		let op = u16::from_ne_bytes(op);

		let Some(object) = client.get_resource_mut(object) else {
			return Err(format!("unknown object '{object}'"))?;
		};

		object.handle(client, op, params)?;
	}
}

fn main() -> Result<()> {
	std::thread::spawn(move || pollster::block_on(render()).unwrap());

	let runtime_dir = std::env::var("XDG_RUNTIME_DIR")?;

	let index = 1;
	let path = std::path::PathBuf::from_iter([runtime_dir, format!("wayland-{index}")]);

	if path.exists() {
		std::fs::remove_file(&path)?;
	}

	let socket = std::os::unix::net::UnixListener::bind(&path)?;

	for (index, stream) in socket.incoming().enumerate() {
		let stream = stream?;
		std::thread::spawn(move || client_event_loop(stream, index).unwrap());
	}

	drop(socket);
	std::fs::remove_file(path)?;

	Ok(())
}
