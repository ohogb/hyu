mod vertex;

use vertex::Vertex;

use winit::platform::{
	scancode::PhysicalKeyExtScancode as _,
	wayland::{EventLoopBuilderExtWayland as _, WindowBuilderExtWayland as _},
};

use crate::{state, wl, Result};

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

pub async fn render() -> Result<()> {
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
	let surface = instance.create_surface(&window)?;

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
				required_features: wgpu::Features::empty(),
				required_limits: wgpu::Limits::downlevel_defaults(),
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
			desired_maximum_frame_latency: 2,
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

	event_loop.run(|event, target| {
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

				let mut clients = state::clients();

				for (client, window) in state::window_stack().iter().rev() {
					let client = clients.get_mut(client).unwrap();
					let window = client.get_object(*window).unwrap();

					let xdg_surface = client.get_object(window.surface).unwrap();
					let surface = client.get_object_mut(xdg_surface.surface).unwrap();

					surface
						.wgpu_do_textures(client, &device, &queue, &sampler, &bind_group_layout)
						.unwrap();

					surface
						.frame(start_time.elapsed().as_millis() as u32, client)
						.unwrap();

					for (x, y, width, height, surface_id) in surface.get_front_buffers(client) {
						let surface = client.get_object(surface_id).unwrap();

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
				let mut should_stop = false;

				let mut clients = state::clients();

				for (client, window) in state::window_stack().iter() {
					let client = clients.get_mut(client).unwrap();

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
							let sub_surface = client.get_object(*child).unwrap();
							let surface = client.get_object(sub_surface.surface).unwrap();

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

					let toplevel = client.get_object(*window).unwrap();
					let xdg_surface = client.get_object(toplevel.surface).unwrap();
					let surface = client.get_object(xdg_surface.surface).unwrap();

					let position = (
						toplevel.position.0 - xdg_surface.position.0,
						toplevel.position.1 - xdg_surface.position.1,
					);

					// let size = xdg_surface.size;

					let Some((w, h, ..)) = surface.data else {
						continue;
					};

					do_stuff(client, surface, cursor_position.into(), position, (w, h));

					if !should_stop {
						for pointer in client.objects_mut::<wl::Pointer>() {
							if old.map(|x| x.0) != client.surface_cursor_is_over.map(|x| x.0) {
								if let Some((old, ..)) = old {
									pointer.leave(client, old).unwrap();
									pointer.frame(client).unwrap();

									for wm_base in client.objects_mut::<wl::XdgWmBase>() {
										wm_base
											.ping(client, start_time.elapsed().as_millis() as u32)
											.unwrap();
									}

									should_stop = true;
								}

								if let Some((surface, (x, y))) = client.surface_cursor_is_over {
									pointer.enter(client, surface, x, y).unwrap();
									pointer.frame(client).unwrap();

									for wm_base in client.objects_mut::<wl::XdgWmBase>() {
										wm_base
											.ping(client, start_time.elapsed().as_millis() as u32)
											.unwrap();
									}

									should_stop = true;
								}
							} else if let Some((_, (x, y))) = client.surface_cursor_is_over {
								pointer.motion(client, x, y).unwrap();
								pointer.frame(client).unwrap();

								for wm_base in client.objects_mut::<wl::XdgWmBase>() {
									wm_base
										.ping(client, start_time.elapsed().as_millis() as u32)
										.unwrap();
								}

								should_stop = true;
							}
						}
					}
				}
			}
			winit::event::WindowEvent::MouseInput { state, button, .. } => match button {
				winit::event::MouseButton::Left => {
					let input_state = match state {
						winit::event::ElementState::Pressed => 1,
						winit::event::ElementState::Released => 0,
					};

					for client in state::clients().values_mut() {
						for pointer in client.objects_mut::<wl::Pointer>() {
							pointer.button(client, 0x110, input_state).unwrap();
							pointer.frame(client).unwrap();
						}
					}
				}
				_ => {}
			},
			winit::event::WindowEvent::KeyboardInput { event, .. } => {
				let code = event.physical_key.to_scancode().unwrap();

				let input_state = match event.state {
					winit::event::ElementState::Pressed => 1,
					winit::event::ElementState::Released => 0,
				};

				let mut clients = state::clients();

				if let Some((client, _window)) = state::window_stack().iter().next() {
					let client = clients.get_mut(client).unwrap();

					for keyboard in client.objects_mut::<wl::Keyboard>() {
						if keyboard.key_states[code as usize] != (input_state != 0) {
							keyboard.key_states[code as usize] = input_state != 0;
							keyboard.key(client, code, input_state).unwrap();
						}
					}
				}
			}
			_ => {}
		}
	})?;

	Ok(())
}
