use glow::HasContext;

use crate::{state, wl, Point, Result};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	pub position: [f32; 2],
	pub uv: [f32; 2],
}

// TODO: this sucks
pub static GLOW: crate::GlobalWrapper<glow::Context> = crate::GlobalWrapper::empty();

pub struct Renderer {
	vertices: Vec<Vertex>,
	width: usize,
	height: usize,
	pub cursor_texture: glow::NativeTexture,
}

impl Renderer {
	pub fn create(glow: glow::Context, width: usize, height: usize) -> Result<Self> {
		unsafe {
			GLOW.initialize(glow);

			(*GLOW.as_mut_ptr()).debug_message_callback(|_, _, _, _, e| {
				eprintln!("{e}");
			});

			GLOW.enable(glow::BLEND);
			GLOW.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

			GLOW.clear_color(0.2, 0.2, 0.2, 1.0);

			let vertex_shader = GLOW
				.create_shader(glow::VERTEX_SHADER)
				.map_err(|x| color_eyre::eyre::eyre!(x))?;

			GLOW.shader_source(vertex_shader, include_str!("vertex_shader.glsl"));
			GLOW.compile_shader(vertex_shader);

			if !GLOW.get_shader_compile_status(vertex_shader) {
				color_eyre::eyre::bail!(GLOW.get_shader_info_log(vertex_shader));
			}

			let fragment_shader = GLOW
				.create_shader(glow::FRAGMENT_SHADER)
				.map_err(|x| color_eyre::eyre::eyre!(x))?;

			GLOW.shader_source(fragment_shader, include_str!("fragment_shader.glsl"));
			GLOW.compile_shader(fragment_shader);

			if !GLOW.get_shader_compile_status(fragment_shader) {
				color_eyre::eyre::bail!(GLOW.get_shader_info_log(fragment_shader));
			}

			let program = GLOW
				.create_program()
				.map_err(|x| color_eyre::eyre::eyre!(x))?;

			GLOW.attach_shader(program, vertex_shader);
			GLOW.attach_shader(program, fragment_shader);

			GLOW.link_program(program);

			if !GLOW.get_program_link_status(program) {
				color_eyre::eyre::bail!(GLOW.get_program_info_log(program));
			}

			GLOW.delete_shader(vertex_shader);
			GLOW.delete_shader(fragment_shader);

			GLOW.use_program(Some(program));

			let vertex_array = GLOW
				.create_vertex_array()
				.map_err(|x| color_eyre::eyre::eyre!(x))?;

			let vertex_buffer = GLOW
				.create_buffer()
				.map_err(|x| color_eyre::eyre::eyre!(x))?;

			GLOW.bind_vertex_array(Some(vertex_array));
			GLOW.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));

			GLOW.vertex_attrib_pointer_f32(
				0,
				2,
				glow::FLOAT,
				false,
				std::mem::size_of::<Vertex>() as _,
				0,
			);

			GLOW.enable_vertex_attrib_array(0);

			GLOW.vertex_attrib_pointer_f32(
				1,
				2,
				glow::FLOAT,
				false,
				std::mem::size_of::<Vertex>() as _,
				std::mem::size_of::<[f32; 2]>() as _,
			);

			GLOW.enable_vertex_attrib_array(1);
		}

		let cursor_texture = unsafe { GLOW.create_texture().unwrap() };

		unsafe {
			GLOW.bind_texture(glow::TEXTURE_2D, Some(cursor_texture));

			GLOW.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MIN_FILTER,
				glow::LINEAR as _,
			);

			GLOW.tex_parameter_i32(
				glow::TEXTURE_2D,
				glow::TEXTURE_MAG_FILTER,
				glow::LINEAR as _,
			);

			let color = [255, 200, 200, 255];

			let arr = [color; (2 * 2) as _];
			let buffer = arr.as_flattened();

			GLOW.tex_image_2d(
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

			GLOW.bind_texture(glow::TEXTURE_2D, None);
		};

		Ok(Renderer {
			vertices: Vec::new(),
			width,
			height,
			cursor_texture,
		})
	}

	pub fn before(&mut self, compositor: &mut state::CompositorState) -> Result<()> {
		unsafe {
			GLOW.clear(glow::COLOR_BUFFER_BIT);
		}

		compositor.render(self)
	}

	pub fn after(
		&mut self,
		compositor: &mut state::CompositorState,
		tv_sec: u32,
		tv_usec: u32,
		sequence: u32,
	) -> Result<()> {
		for (fd, xdg_toplevel) in compositor.windows.iter().map(|x| **x) {
			let client = compositor.clients.get_mut(&fd).unwrap();
			let display = client.get_object(wl::Id::<wl::Display>::new(1))?;

			let frame = |client: &mut wl::Client, surface: &mut wl::Surface| -> Result<()> {
				surface.frame(display.get_time().as_millis() as u32, client)?;
				surface.presentation_feedback(
					std::time::Duration::from_micros(tv_sec as u64 * 1_000_000 + tv_usec as u64),
					0,
					sequence as _,
					0x2,
					client,
				)
			};

			let toplevel = client.get_object(xdg_toplevel)?;
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

	pub fn quad(&mut self, position: Point, size: Point, texture: &glow::NativeTexture) {
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
			GLOW.buffer_data_u8_slice(
				glow::ARRAY_BUFFER,
				bytemuck::cast_slice(&self.vertices),
				glow::DYNAMIC_DRAW,
			);

			GLOW.bind_texture(glow::TEXTURE_2D, Some(*texture));

			GLOW.draw_arrays(glow::TRIANGLES, (self.vertices.len() - 6) as _, 6);
		}
	}
}
