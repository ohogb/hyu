use glow::HasContext;
use glutin::{context::NotCurrentGlContext, display::GlDisplay, surface::GlSurface};
use raw_window_handle::{HasRawDisplayHandle as _, HasRawWindowHandle};

use crate::{backend, Result};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	pub position: [f32; 3],
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

			let vertices = &[
				Vertex {
					position: [-0.5, -0.5, 0.0],
				},
				Vertex {
					position: [0.5, -0.5, 0.0],
				},
				Vertex {
					position: [0.0, 0.5, 0.0],
				},
			];

			glow.buffer_data_u8_slice(
				glow::ARRAY_BUFFER,
				bytemuck::cast_slice(vertices),
				glow::DYNAMIC_DRAW,
			);

			glow.vertex_attrib_pointer_f32(
				0,
				3,
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
			})
		}
	}
}

struct Renderer<'a> {
	window: &'a winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	glow: glow::Context,
}

impl<'a> backend::winit::WinitRenderer for Renderer<'a> {
	fn render(&mut self) -> Result<()> {
		self.window.request_redraw();

		unsafe {
			self.glow.clear(glow::COLOR_BUFFER_BIT);
			self.glow.draw_arrays(glow::TRIANGLES, 0, 3);
		}

		self.surface.swap_buffers(&self.context)?;
		Ok(())
	}
}
