use glow::HasContext;
use glutin::{context::NotCurrentGlContext, display::GlDisplay, surface::GlSurface};
use raw_window_handle::{HasRawDisplayHandle as _, HasRawWindowHandle};

use crate::{backend, Result};

pub struct Setup;

impl backend::winit::WinitRendererSetup for Setup {
	fn setup(&self, window: &winit::window::Window) -> Result<impl backend::winit::WinitRenderer> {
		let (surface, context, glow) = unsafe {
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
						std::num::NonZeroU32::new(1280).unwrap(),
						std::num::NonZeroU32::new(720).unwrap(),
					),
			)?;

			let context = context.make_current(&surface)?;

			let glow = glow::Context::from_loader_function_cstr(|x| display.get_proc_address(x));
			(surface, context, glow)
		};

		Ok(Renderer {
			window,
			surface,
			context,
			glow,
		})
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
			self.glow.clear_color(0.2, 0.2, 0.2, 1.0);
		}

		self.surface.swap_buffers(&self.context)?;
		Ok(())
	}
}
