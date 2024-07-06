use glow::HasContext;
use glutin::{
	context::NotCurrentGlContext,
	display::{AsRawDisplay, GlDisplay},
	surface::GlSurface,
};
use raw_window_handle::{HasRawDisplayHandle as _, HasRawWindowHandle};

use crate::{backend, state, wl, Point, Result};

pub mod egl_wrapper;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	pub position: [f32; 2],
	pub uv: [f32; 2],
}

pub struct Renderer {
	glow: glow::Context,
	vertices: Vec<Vertex>,
	start_time: std::time::Instant,
	width: usize,
	height: usize,
	cursor_texture: glow::NativeTexture,
}

impl Renderer {
	pub fn create(mut glow: glow::Context, width: usize, height: usize) -> Result<Self> {
		unsafe {
			glow.debug_message_callback(|_, _, _, _, e| {
				eprintln!("{e}");
			});

			glow.enable(glow::BLEND);
			glow.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

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

			glow.vertex_attrib_pointer_f32(
				1,
				2,
				glow::FLOAT,
				false,
				std::mem::size_of::<Vertex>() as _,
				std::mem::size_of::<[f32; 2]>() as _,
			);

			glow.enable_vertex_attrib_array(1);
		}

		let cursor_texture = unsafe { glow.create_texture().unwrap() };

		unsafe {
			glow.bind_texture(glow::TEXTURE_2D, Some(cursor_texture));

			glow.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR as _,
			);

			glow.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MAG_FILTER,
				glow::LINEAR as _,
			);

			let buffer = &[255; 2 * 2 * 4];

			glow.tex_image_2d(
				glow::TEXTURE_2D,
				0,
				glow::RGBA as _,
				2,
				2,
				0,
				glow::BGRA,
				glow::UNSIGNED_BYTE,
				Some(buffer),
			);

			glow.bind_texture(glow::TEXTURE_2D, None);
		};

		Ok(Renderer {
			glow,
			vertices: Vec::new(),
			start_time: std::time::Instant::now(),
			width,
			height,
			cursor_texture,
		})
	}

	pub fn before(&mut self) -> Result<()> {
		unsafe {
			self.glow.clear(glow::COLOR_BUFFER_BIT);
		}

		let mut clients = state::CLIENTS.lock().unwrap();

		for (client, window) in state::WINDOW_STACK.lock().unwrap().iter().rev() {
			let client = clients.get_mut(client).unwrap();

			fn draw(
				this: &mut Renderer,
				client: &mut wl::Client,
				toplevel_position: Point,
				xdg_surface: &wl::XdgSurface,
				surface: &mut wl::Surface,
			) -> Result<()> {
				surface.gl_do_textures(client, &this.glow)?;

				for (position, size, surface_id) in surface.get_front_buffers(client) {
					let surface = client.get_object(surface_id)?;

					let Some((.., wl::SurfaceTexture::Gl(texture))) = &surface.data else {
						panic!();
					};

					this.quad(
						toplevel_position - xdg_surface.position + position,
						size,
						texture,
					);
				}

				Ok(())
			}

			let toplevel = client.get_object(*window)?;

			let xdg_surface = client.get_object(toplevel.surface)?;
			let surface = client.get_object_mut(xdg_surface.surface)?;

			draw(self, client, toplevel.position, xdg_surface, surface)?;

			for &popup in &xdg_surface.popups {
				let popup = client.get_object(popup)?;

				let xdg_surface = client.get_object(popup.xdg_surface)?;
				let surface = client.get_object_mut(xdg_surface.surface)?;

				let position = toplevel.position + popup.position;

				draw(self, client, position, xdg_surface, surface)?;
			}
		}

		let cursor_pos = state::POINTER_POSITION.lock().unwrap().clone();
		self.quad(cursor_pos, Point(2, 2), &self.cursor_texture.clone());

		Ok(())
	}

	pub fn after(&mut self) -> Result<()> {
		let time = nix::time::clock_gettime(nix::time::ClockId::CLOCK_MONOTONIC)?;
		let mut clients = state::CLIENTS.lock().unwrap();

		for (client, window) in state::WINDOW_STACK.lock().unwrap().iter().rev() {
			let client = clients.get_mut(client).unwrap();

			let frame = |client: &mut wl::Client, surface: &mut wl::Surface| -> Result<()> {
				surface.frame(self.start_time.elapsed().as_millis() as u32, client)?;
				surface.presentation_feedback(time, 0, 0, 0, client)
			};

			let toplevel = client.get_object(*window)?;
			let xdg_surface = client.get_object(toplevel.surface)?;
			let surface = client.get_object_mut(xdg_surface.surface)?;

			frame(client, surface)?;

			for &popup in &xdg_surface.popups {
				let popup = client.get_object(popup)?;

				let xdg_surface = client.get_object(popup.xdg_surface)?;
				let surface = client.get_object_mut(xdg_surface.surface)?;

				frame(client, surface)?;
			}
		}

		self.vertices.clear();
		Ok(())
	}

	fn quad(&mut self, position: Point, size: Point, texture: &glow::NativeTexture) {
		let pixels_to_float = |input: [i32; 2]| -> [f32; 2] {
			[
				input[0] as f32 / self.width as f32 * 2.0 - 1.0,
				(input[1] as f32 / self.height as f32 * 2.0 - 1.0) * -1.0,
			]
		};

		let Point(x, y) = position;
		let Point(width, height) = size;

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

		unsafe {
			self.glow.buffer_data_u8_slice(
				glow::ARRAY_BUFFER,
				bytemuck::cast_slice(&self.vertices),
				glow::DYNAMIC_DRAW,
			);

			self.glow.bind_texture(glow::TEXTURE_2D, Some(*texture));

			self.glow
				.draw_arrays(glow::TRIANGLES, (self.vertices.len() - 6) as _, 6);
		}
	}
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
					.with_context_api(glutin::context::ContextApi::Gles(Some(
						glutin::context::Version::new(3, 2),
					)))
					.with_debug(true)
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

			let raw_display = match display.raw_display() {
				glutin::display::RawDisplay::Egl(x) => x,
			};

			egl_wrapper::init(raw_display as _, |name| {
				let name_as_cstring = std::ffi::CString::new(name)?;
				let ret = display.get_proc_address(name_as_cstring.as_c_str());

				if ret.is_null() {
					Err(format!("cannot find function '{name}'"))?;
				}

				Ok(ret as _)
			})?;

			Ok(WinitRenderer {
				window,
				surface,
				context,
				renderer: Renderer::create(glow, width, height)?,
			})
		}
	}
}

struct WinitRenderer<'a> {
	window: &'a winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	renderer: Renderer,
}

impl<'a> backend::winit::WinitRenderer for WinitRenderer<'a> {
	fn render(&mut self) -> Result<()> {
		self.renderer.before()?;
		self.surface.swap_buffers(&self.context)?;
		self.renderer.after()?;

		self.window.request_redraw();
		Ok(())
	}
}
