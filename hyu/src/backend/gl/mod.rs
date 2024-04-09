use glow::HasContext;
use glutin::{context::NotCurrentGlContext, display::GlDisplay, surface::GlSurface};
use raw_window_handle::{HasRawDisplayHandle as _, HasRawWindowHandle};

use crate::{backend, state, wl, Result};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	pub position: [f32; 2],
}

pub struct Setup;

impl backend::winit::WinitRendererSetup for Setup {
	fn setup(
		&self,
		window: &winit::window::Window,
		width: usize,
		height: usize,
	) -> Result<impl backend::winit::WinitRenderer> {
		unsafe {
			let display = glutin::display::Display::new(
				window.raw_display_handle(),
				glutin::display::DisplayApiPreference::Egl,
			)?;

			let config = display
				.find_configs(glutin::config::ConfigTemplateBuilder::new().build())?
				.next()
				.unwrap();

			let context = display.create_context(
				&config,
				&glutin::context::ContextAttributesBuilder::new()
					.build(Some(window.raw_window_handle())),
			)?;

			let surface = display.create_window_surface(
				&config,
				&glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
					.build(
						window.raw_window_handle(),
						std::num::NonZeroU32::new(width as _).unwrap(),
						std::num::NonZeroU32::new(height as _).unwrap(),
					),
			)?;

			let context = context.make_current(&surface)?;

			surface.set_swap_interval(
				&context,
				glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap()),
			)?;

			let glow = glow::Context::from_loader_function_cstr(|x| display.get_proc_address(x));

			glow.clear_color(0.2, 0.2, 0.2, 1.0);

			let vertex_shader = glow.create_shader(glow::VERTEX_SHADER)?;

			glow.shader_source(vertex_shader, include_str!("vertex_shader.glsl"));
			glow.compile_shader(vertex_shader);

			if !glow.get_shader_compile_status(vertex_shader) {
				Err(glow.get_shader_info_log(vertex_shader))?
			}

			let fragment_shader = glow.create_shader(glow::FRAGMENT_SHADER)?;

			glow.shader_source(fragment_shader, include_str!("fragment_shader.glsl"));
			glow.compile_shader(fragment_shader);

			if !glow.get_shader_compile_status(fragment_shader) {
				Err(glow.get_shader_info_log(fragment_shader))?
			}

			let program = glow.create_program()?;

			glow.attach_shader(program, vertex_shader);
			glow.attach_shader(program, fragment_shader);

			glow.link_program(program);

			if !glow.get_program_link_status(program) {
				Err(glow.get_program_info_log(program))?
			}

			glow.delete_shader(vertex_shader);
			glow.delete_shader(fragment_shader);

			glow.use_program(Some(program));

			let vertex_array = glow.create_vertex_array()?;
			let vertex_buffer = glow.create_buffer()?;

			glow.bind_vertex_array(Some(vertex_array));
			glow.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));

			glow.vertex_attrib_pointer_f32(
				0,
				2,
				glow::FLOAT,
				false,
				std::mem::size_of::<Vertex>() as _,
				0,
			);

			glow.enable_vertex_attrib_array(0);

			Ok(Renderer {
				window,
				surface,
				context,
				glow,
				width,
				height,
			})
		}
	}
}

struct Renderer<'a> {
	window: &'a winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	glow: glow::Context,
	width: usize,
	height: usize,
}

impl<'a> backend::winit::WinitRenderer for Renderer<'a> {
	fn render(&mut self) -> Result<()> {
		self.window.request_redraw();

		unsafe {
			self.glow.clear(glow::COLOR_BUFFER_BIT);
		}

		let mut clients = state::clients();

		let mut vertices = Vec::new();

		for (client, window) in state::window_stack().iter().rev() {
			let client = clients.get_mut(client).unwrap();
			let window = client.get_object(*window)?;

			let xdg_surface = client.get_object(window.surface)?;
			let surface = client.get_object_mut(xdg_surface.surface)?;

			surface.gl_do_textures(client)?;

			surface.frame(0, client)?;

			for (x, y, width, height, surface_id) in surface.get_front_buffers(client) {
				let surface = client.get_object(surface_id)?;

				let Some((.., wl::SurfaceTexture::Gl())) = &surface.data else {
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

				vertices.push(Vertex {
					position: pixels_to_float([x, y]),
				});

				vertices.push(Vertex {
					position: pixels_to_float([x + width, y]),
				});

				vertices.push(Vertex {
					position: pixels_to_float([x, y + height]),
				});

				vertices.push(Vertex {
					position: pixels_to_float([x, y + height]),
				});

				vertices.push(Vertex {
					position: pixels_to_float([x + width, y + height]),
				});

				vertices.push(Vertex {
					position: pixels_to_float([x + width, y]),
				});

				unsafe {
					self.glow.buffer_data_u8_slice(
						glow::ARRAY_BUFFER,
						bytemuck::cast_slice(&vertices),
						glow::DYNAMIC_DRAW,
					);

					self.glow
						.draw_arrays(glow::TRIANGLES, (vertices.len() - 6) as _, 6);
				}
			}
		}

		self.surface.swap_buffers(&self.context)?;
		Ok(())
	}
}
