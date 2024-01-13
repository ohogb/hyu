#![feature(fs_try_exists, unix_socket_peek)]

mod state;
pub mod wl;

pub use state::*;
use winit::platform::wayland::{EventLoopBuilderExtWayland, WindowBuilderExtWayland};
use wl::Object;

use std::{
	io::{Read, Write},
	os::fd::AsRawFd,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 2],
	color: [f32; 4],
}

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
	let runtime_dir = std::env::var("XDG_RUNTIME_DIR")?;

	let index = 1;
	let path = std::path::PathBuf::from_iter([runtime_dir, format!("wayland-{index}")]);

	if std::fs::try_exists(&path)? {
		std::fs::remove_file(&path)?;
	}

	let socket = std::os::unix::net::UnixListener::bind(&path)?;

	let clients = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::<
		std::os::fd::RawFd,
		wl::Client,
	>::new()));

	let buffer = std::sync::Arc::new(std::sync::Mutex::new(Vec::with_capacity(
		WIDTH * HEIGHT * 8,
	)));

	let ptr = clients.as_ref() as *const _ as u64;

	let bufferb = buffer.clone();
	std::thread::spawn(move || {
		let buffer = bufferb;

		pollster::block_on(async {
			env_logger::init();

			let mut event_loop_builder = winit::event_loop::EventLoopBuilder::new();
			event_loop_builder.with_any_thread(true);

			let event_loop = event_loop_builder.build()?;
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
				source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
					"shader.wgsl"
				))),
			});

			let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[],
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
					present_mode: wgpu::PresentMode::Mailbox,
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

			let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: None,
				layout: Some(&pipeline_layout),
				vertex: wgpu::VertexState {
					module: &shader,
					entry_point: "vs_main",
					buffers: &[wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<Vertex>() as _,
						step_mode: wgpu::VertexStepMode::Vertex,
						attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
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
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::PointList,
					..Default::default()
				},
				depth_stencil: None,
				multisample: wgpu::MultisampleState::default(),
				multiview: None,
			});

			event_loop.run(move |event, target| {
				let winit::event::Event::WindowEvent { window_id, event } = event else {
					return;
				};

				if window_id != window.id() {
					return;
				}

				match event {
					winit::event::WindowEvent::RedrawRequested => {
						let buffer = buffer.lock().unwrap();
						queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&buffer));

						let frame = surface.get_current_texture().unwrap();

						let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
							..Default::default()
						});

						let mut encoder =
							device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
								label: None,
							});

						{
							let mut render_pass =
								encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
									label: None,
									color_attachments: &[Some(wgpu::RenderPassColorAttachment {
										view: &view,
										resolve_target: None,
										ops: wgpu::Operations {
											load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
											store: wgpu::StoreOp::Store,
										},
									})],
									depth_stencil_attachment: None,
									timestamp_writes: None,
									occlusion_query_set: None,
								});

							render_pass.set_pipeline(&render_pipeline);
							render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
							render_pass.draw(0..buffer.len() as _, 0..1);
						}

						queue.submit(Some(encoder.finish()));
						frame.present();

						window.request_redraw();
					}
					winit::event::WindowEvent::CloseRequested => {
						target.exit();
					}
					_ => {}
				}
			})?;

			Result::Ok(())
		})
		.unwrap()
	});

	for (index, stream) in socket.incoming().enumerate() {
		let mut stream = stream?;

		let buffer = buffer.clone();

		std::thread::spawn(move || {
			|| -> Result<()> {
				let ptr = ptr as *const std::sync::Mutex<
					std::collections::HashMap<std::os::fd::RawFd, wl::Client>,
				>;

				let mut client = wl::Client::new(State {
					buffer: Buffer(Vec::new()),
					start_position: (100 * (index + 1) as i32, 100 * (index + 1) as i32),
				});

				let mut display = wl::Display::new();

				display.push_global(wl::Shm::new());
				display.push_global(wl::Compositor::new());
				display.push_global(wl::SubCompositor::new());
				display.push_global(wl::DataDeviceManager::new());
				display.push_global(wl::Seat::new());
				display.push_global(wl::Output::new());
				display.push_global(wl::XdgWmBase::new());

				client.push_client_object(1, display);

				unsafe {
					let mut lock = (*ptr).lock().unwrap();
					lock.insert(stream.as_raw_fd(), client);
				}

				loop {
					let fd = stream.as_raw_fd();

					let mut cmsg = nix::cmsg_space!([std::os::fd::RawFd; 10]);

					let msgs = nix::sys::socket::recvmsg::<()>(
						fd,
						&mut [],
						Some(&mut cmsg),
						nix::sys::socket::MsgFlags::empty(),
					)?;

					let mut ptr = unsafe { (*ptr).lock().unwrap() };
					let client = ptr.get_mut(&stream.as_raw_fd()).unwrap();

					for i in msgs.cmsgs() {
						match i {
							nix::sys::socket::ControlMessageOwned::ScmRights(x) => {
								client.push_fds(x)
							}
							_ => panic!(),
						}
					}

					let mut obj = [0u8; 4];
					stream.read_exact(&mut obj).unwrap();

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

					let Some(object) = client.get_object_mut(object) else {
						return Err(format!("unknown object '{object}'"))?;
					};

					// TODO: think how to do this the safe way
					let object = object as *mut wl::Resource;
					unsafe { (*object).handle(client, op, params)? };

					stream.write_all(&client.get_state().buffer.0)?;
					client.get_state().buffer.0.clear();

					let mut buffer = buffer.lock().unwrap();
					buffer.clear();

					for client in ptr.values_mut() {
						for window in client.get_windows() {
							let wl::Resource::XdgToplevel(window) = window else {
								panic!();
							};

							let Some(wl::Resource::XdgSurface(xdg_surface)) =
								client.get_object(window.surface)
							else {
								panic!();
							};

							let pos = xdg_surface.position;

							let Some(wl::Resource::Surface(surface)) =
								client.get_object(xdg_surface.get_surface())
							else {
								panic!();
							};

							for (x, y, width, _height, bytes_per_pixel, pixels) in
								surface.get_front_buffers(client)
							{
								for (index, pixel) in
									pixels.chunks(bytes_per_pixel as _).enumerate()
								{
									let index = index as i32;
									let position = window.position;

									let x = (index % width) + position.0 - pos.0 + x;
									let y = (index / width) + position.1 - pos.1 + y;

									buffer.push(Vertex {
										position: [
											x as f32 / WIDTH as f32 * 2.0 - 1.0,
											(y as f32 / HEIGHT as f32 * 2.0 - 1.0) * -1.0,
										],
										color: [
											((pixel[2] as f32 / 255.0 + 0.055) / 1.055).powf(2.4),
											((pixel[1] as f32 / 255.0 + 0.055) / 1.055).powf(2.4),
											((pixel[0] as f32 / 255.0 + 0.055) / 1.055).powf(2.4),
											((pixel[3] as f32 / 255.0 + 0.055) / 1.055).powf(2.4),
										],
									});
								}
							}
						}
					}
				}
			}()
			.unwrap();
		});
	}

	drop(socket);
	std::fs::remove_file(path)?;

	Ok(())
}
