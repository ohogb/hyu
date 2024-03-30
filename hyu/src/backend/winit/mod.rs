use winit::platform::{
	scancode::PhysicalKeyExtScancode as _,
	wayland::{EventLoopBuilderExtWayland as _, WindowBuilderExtWayland as _},
};

use crate::{state, wl, Result};

pub trait WinitRendererSetup {
	fn setup(&self, window: &winit::window::Window) -> Result<impl WinitRenderer>;
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

	let mut renderer = renderer_setup.setup(&window)?;

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
				let mut clients = state::clients();

				let old = {
					let mut lock = state::pointer_over();
					let ret = *lock;

					*lock = None;
					ret
				};

				for (client, window) in state::window_stack().iter() {
					if state::pointer_over().is_some() {
						break;
					}

					let client = clients.get_mut(client).unwrap();

					fn get_surface_input_size(
						client: &wl::Client,
						surface: &wl::Surface,
					) -> (i32, i32) {
						None.or_else(|| {
							if surface.current_input_region.as_ref()?.areas.is_empty() {
								Some((0, 0))
							} else {
								None
							}
						})
						.or_else(|| {
							let &(w, h, ..) = surface.data.as_ref()?;
							Some((w, h))
						})
						.unwrap_or((0, 0))
					}

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
							*state::pointer_over() = Some((
								client.fd,
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

							do_stuff(
								client,
								surface,
								cursor_position,
								(
									surface_position.0 + sub_surface.position.0,
									surface_position.1 + sub_surface.position.1,
								),
								get_surface_input_size(client, surface),
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

					do_stuff(
						client,
						surface,
						cursor_position.into(),
						position,
						get_surface_input_size(client, surface),
					);
				}

				let current = *state::pointer_over();

				if old.is_none() && current.is_none() {
					return;
				}

				if old.map(|x| (x.0, x.1)) != current.map(|x| (x.0, x.1)) {
					if let Some((fd, surface, ..)) = old {
						let client = clients.get_mut(&fd).unwrap();

						for pointer in client.objects_mut::<wl::Pointer>() {
							pointer.leave(client, surface).unwrap();
							pointer.frame(client).unwrap();
						}
					}

					if let Some((fd, surface, (x, y))) = current {
						let client = clients.get_mut(&fd).unwrap();

						for pointer in client.objects_mut::<wl::Pointer>() {
							pointer.enter(client, surface, x, y).unwrap();
							pointer.frame(client).unwrap();
						}
					}
				} else if old.map(|x| x.2) != current.map(|x| x.2) {
					let (fd, _, (x, y)) = current.unwrap();
					let client = clients.get_mut(&fd).unwrap();

					for pointer in client.objects_mut::<wl::Pointer>() {
						pointer.motion(client, x, y).unwrap();
						pointer.frame(client).unwrap();
					}
				}
			}
			winit::event::WindowEvent::MouseInput { state, button, .. } => match button {
				winit::event::MouseButton::Left => {
					let input_state = match state {
						winit::event::ElementState::Pressed => 1,
						winit::event::ElementState::Released => 0,
					};

					if let Some((fd, ..)) = *state::pointer_over() {
						let mut clients = state::clients();
						let client = clients.get_mut(&fd).unwrap();

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
