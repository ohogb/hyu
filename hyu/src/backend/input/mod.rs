use color_eyre::eyre::OptionExt as _;

use crate::{Result, elp, libinput, state, udev};

pub struct State {
	x: f64,
	y: f64,
}

pub fn initialize_state() -> Result<State> {
	Ok(State { x: 0.0, y: 0.0 })
}

pub fn attach(
	event_loop: &mut elp::EventLoop<state::State>,
	_state: &mut state::State,
) -> Result<()> {
	let udev = udev::Instance::create().ok_or_eyre("failed to create udev instance")?;
	let context = libinput::Context::create_from_udev(udev)
		.ok_or_eyre("failed to create libinput context")?;

	let ret = context.assign();
	assert!(ret != -1);

	event_loop.on(elp::input::create(context), |msg, state, _| {
		let elp::input::Message::Event { event } = msg;

		match event.get_type() {
			300 => {
				let Some(keyboard) = event.get_keyboard_event() else {
					panic!();
				};

				state
					.compositor
					.on_keyboard_button(keyboard.get_key(), keyboard.get_key_state() as _)?;
			}
			400 => {
				let Some(pointer) = event.get_pointer_event() else {
					panic!();
				};

				state.hw.input.x += pointer.get_dx();
				state.hw.input.y += pointer.get_dy();

				state.hw.input.x = state
					.hw
					.input
					.x
					.clamp(0.0, (state.compositor.width - 1) as f64);

				state.hw.input.y = state
					.hw
					.input
					.y
					.clamp(0.0, (state.compositor.height - 1) as f64);

				state
					.compositor
					.on_cursor_move((state.hw.input.x as _, state.hw.input.y as _))?;
			}
			402 => {
				let Some(pointer) = event.get_pointer_event() else {
					panic!();
				};

				let button = pointer.get_button();
				let button_state = pointer.get_button_state();

				state.compositor.on_mouse_button(button, button_state)?;
			}
			404 => {
				let Some(pointer) = event.get_pointer_event() else {
					panic!();
				};

				let v120 = pointer.get_scroll_value_v120(0);
				state
					.compositor
					.on_mouse_scroll(v120 / 12.0, (v120 / 120.0) as _, 0)?;
			}
			_ => {}
		}

		Ok(())
	})
}
