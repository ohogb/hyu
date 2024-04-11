mod vertex;

use vertex::Vertex;

use crate::{backend, state, wl, Result};

pub struct Setup;

impl backend::winit::WinitRendererSetup for Setup {
	fn setup(
		&self,
		window: &winit::window::Window,
		width: usize,
		height: usize,
	) -> Result<impl backend::winit::WinitRenderer> {
		let instance = wgpu::Instance::default();
		let surface = instance.create_surface(window)?;

		let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
			compatible_surface: Some(&surface),
			..Default::default()
		}))
		.unwrap();

		let (device, queue) = pollster::block_on(adapter.request_device(
			&wgpu::DeviceDescriptor {
				label: None,
				required_features: wgpu::Features::empty(),
				required_limits: wgpu::Limits::downlevel_defaults(),
			},
			None,
		))?;

		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
				"shader.wgsl"
			))),
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
			size: (std::mem::size_of::<Vertex>() * width * height * 8) as u64,
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

		Ok(Renderer {
			window,
			surface,
			device,
			queue,
			bind_group_layout,
			vertex_buffer,
			sampler,
			render_pipeline,
			vertices: Vec::with_capacity(width * height),
			start_time: std::time::Instant::now(),
			width,
			height,
		})
	}
}

pub struct Renderer<'a> {
	window: &'a winit::window::Window,
	surface: wgpu::Surface<'a>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	bind_group_layout: wgpu::BindGroupLayout,
	vertex_buffer: wgpu::Buffer,
	sampler: wgpu::Sampler,
	render_pipeline: wgpu::RenderPipeline,
	vertices: Vec<Vertex>,
	start_time: std::time::Instant,
	width: usize,
	height: usize,
}

impl<'a> backend::winit::WinitRenderer for Renderer<'a> {
	fn render(&mut self) -> Result<()> {
		let frame = self.surface.get_current_texture()?;

		let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
			..Default::default()
		});

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

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
			let window = client.get_object(*window)?;

			let xdg_surface = client.get_object(window.surface)?;
			let surface = client.get_object_mut(xdg_surface.surface)?;

			surface.wgpu_do_textures(
				client,
				&self.device,
				&self.queue,
				&self.sampler,
				&self.bind_group_layout,
			)?;

			surface.frame(self.start_time.elapsed().as_millis() as u32, client)?;

			for (x, y, width, height, surface_id) in surface.get_front_buffers(client) {
				let surface = client.get_object(surface_id)?;

				let Some((.., wl::SurfaceTexture::Wgpu(_, bind_group))) = &surface.data else {
					panic!();
				};

				let pixels_to_float = |input: [i32; 2]| -> [f32; 2] {
					[
						input[0] as f32 / self.width as f32 * 2.0 - 1.0,
						(input[1] as f32 / self.height as f32 * 2.0 - 1.0) * -1.0,
					]
				};

				let x = window.position.0 - xdg_surface.position.0 + x;
				let y = window.position.1 - xdg_surface.position.1 + y;

				self.vertices.extend([
					Vertex {
						position: pixels_to_float([x, y]),
						uv: [0.0, 0.0],
					},
					Vertex {
						position: pixels_to_float([x + width, y]),
						uv: [1.0, 0.0],
					},
					Vertex {
						position: pixels_to_float([x, y + height]),
						uv: [0.0, 1.0],
					},
					Vertex {
						position: pixels_to_float([x, y + height]),
						uv: [0.0, 1.0],
					},
					Vertex {
						position: pixels_to_float([x + width, y + height]),
						uv: [1.0, 1.0],
					},
					Vertex {
						position: pixels_to_float([x + width, y]),
						uv: [1.0, 0.0],
					},
				]);

				let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

				render_pass.set_pipeline(&self.render_pipeline);
				render_pass.set_bind_group(0, bind_group, &[]);
				render_pass.set_vertex_buffer(
					0,
					self.vertex_buffer.slice(
						((self.vertices.len() - 6) * std::mem::size_of::<Vertex>()) as u64..,
					),
				);
				render_pass.draw(0..6 as _, 0..1);
			}
		}

		self.queue
			.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));

		self.queue.submit(Some(encoder.finish()));
		frame.present();

		self.vertices.clear();
		self.window.request_redraw();

		Ok(())
	}
}
