use winit::platform::{
	scancode::PhysicalKeyExtScancode as _,
	wayland::{EventLoopBuilderExtWayland as _, WindowBuilderExtWayland as _},
};

use crate::{state, Result};

pub trait WinitRendererSetup {
	fn setup(
		&self,
		window: &winit::window::Window,
		width: usize,
		height: usize,
	) -> Result<impl WinitRenderer>;
}

pub trait WinitRenderer {
	fn render(&mut self) -> Result<()>;
}

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

pub fn run(renderer_setup: impl WinitRendererSetup) -> Result<()> {
	let event_loop = winit::event_loop::EventLoopBuilder::new()
		.with_any_thread(true)
		.build()?;

	let window = winit::window::WindowBuilder::new()
		.with_name("hyu", "hyu")
		.with_inner_size(winit::dpi::PhysicalSize::new(WIDTH as u32, HEIGHT as u32))
		.with_fullscreen(None)
		.build(&event_loop)?;

	let mut renderer = renderer_setup.setup(&window, WIDTH, HEIGHT)?;

	event_loop.run(|event, target| {
		let winit::event::Event::WindowEvent { window_id, event } = event else {
			return;
		};

		if window_id != window.id() {
			return;
		}

		match event {
			winit::event::WindowEvent::RedrawRequested => {
				renderer.render().unwrap();
			}
			winit::event::WindowEvent::CloseRequested => {
				target.exit();
			}
			winit::event::WindowEvent::CursorMoved {
				position: cursor_position,
				..
			} => {
				state::on_cursor_move(cursor_position.into()).unwrap();
			}
			winit::event::WindowEvent::MouseInput { state, button, .. } => match button {
				winit::event::MouseButton::Left => {
					let input_state = match state {
						winit::event::ElementState::Pressed => 1,
						winit::event::ElementState::Released => 0,
					};

					state::on_mouse_button_left(input_state).unwrap();
				}
				_ => {}
			},
			winit::event::WindowEvent::KeyboardInput { event, .. } => {
				let code = event.physical_key.to_scancode().unwrap();

				let input_state = match event.state {
					winit::event::ElementState::Pressed => 1,
					winit::event::ElementState::Released => 0,
				};

				state::on_keyboard_button(code, input_state).unwrap();
			}
			_ => {}
		}
	})?;

	Ok(())
}
