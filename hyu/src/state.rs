use std::{
	io::{Seek as _, Write as _},
	os::fd::{FromRawFd as _, IntoRawFd as _},
};

use crate::{wl, xkb, Point, Result};

pub static CLIENTS: std::sync::LazyLock<
	std::sync::Mutex<std::collections::HashMap<std::os::fd::RawFd, wl::Client>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

pub static WINDOW_STACK: std::sync::LazyLock<
	std::sync::Mutex<std::collections::VecDeque<(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>)>>,
> = std::sync::LazyLock::new(Default::default);

pub enum Change {
	Push(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
	RemoveToplevel(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
	RemoveSurface(std::os::fd::RawFd, wl::Id<wl::Surface>),
	RemoveClient(std::os::fd::RawFd),
	Pick(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
}

pub static CHANGES: std::sync::LazyLock<std::sync::Mutex<Vec<Change>>> =
	std::sync::LazyLock::new(Default::default);

#[derive(Clone, Copy)]
pub struct PointerOver {
	pub fd: std::os::fd::RawFd,
	pub toplevel: wl::Id<wl::XdgToplevel>,
	pub surface: wl::Id<wl::Surface>,
	pub position: Point,
}

pub static POINTER_OVER: std::sync::LazyLock<std::sync::Mutex<Option<PointerOver>>> =
	std::sync::LazyLock::new(Default::default);

pub static POINTER_POSITION: std::sync::Mutex<Point> = std::sync::Mutex::new(Point(0, 0));

pub struct XkbState {
	pub _context: xkb::Context,
	pub _keymap: xkb::Keymap,
	pub state: xkb::State,
	pub keymap_file: (std::os::fd::RawFd, u64),
}

pub static XKB_STATE: std::sync::Mutex<Option<XkbState>> = std::sync::Mutex::new(None);

pub fn initialize_xkb_state(layout: impl AsRef<str>) -> Result<()> {
	let xkb_context = xkb::Context::create().ok_or("failed to create xkb context")?;

	let xkb_keymap =
		xkb::Keymap::create(&xkb_context, layout).ok_or("failed to create xkb keymap")?;

	let xkb_state = xkb::State::new(&xkb_keymap).ok_or("failed to create xkb state")?;

	let (fd, path) = nix::unistd::mkstemp("/tmp/temp_XXXXXX")?;
	nix::unistd::unlink(&path)?;

	let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
	write!(file, "{}", xkb_keymap.get_as_string())?;

	let size = file.stream_len()?;
	let fd = file.into_raw_fd();

	*XKB_STATE.lock().unwrap() = Some(XkbState {
		_context: xkb_context,
		_keymap: xkb_keymap,
		state: xkb_state,
		keymap_file: (fd, size),
	});

	Ok(())
}

pub fn get_xkb_keymap() -> (std::os::fd::RawFd, u64) {
	let lock = XKB_STATE.lock().unwrap();

	let Some(xkb_state) = &*lock else {
		panic!();
	};

	xkb_state.keymap_file
}

pub fn process_focus_changes(
	clients: &mut std::sync::MutexGuard<
		'_,
		std::collections::HashMap<std::os::fd::RawFd, wl::Client>,
	>,
) -> Result<()> {
	let mut lock = WINDOW_STACK.lock().unwrap();
	let old = lock.iter().next().cloned();

	let mut should_leave_from_old = false;
	let mut should_recompute_size_and_pos = false;

	for (i, change) in std::mem::take(&mut *CHANGES.lock().unwrap())
		.into_iter()
		.enumerate()
	{
		should_recompute_size_and_pos = true;

		let x = match change {
			Change::Push(fd, id) => {
				lock.push_front((fd, id));

				let client = clients.get_mut(&fd).unwrap();

				let toplevel = client.get_object_mut(id)?;
				toplevel.add_state(1);

				true
			}
			Change::RemoveToplevel(fd, id) => {
				lock.retain(|&x| x != (fd, id));

				let mut lock = POINTER_OVER.lock().unwrap();

				if let Some(value) = &mut *lock {
					if value.fd == fd && value.toplevel == id {
						*lock = None;
					}
				}

				false
			}
			Change::RemoveSurface(fd, id) => {
				let mut lock = POINTER_OVER.lock().unwrap();

				if let Some(value) = &mut *lock {
					if value.fd == fd && value.surface == id {
						*lock = None;
					}
				}

				false
			}
			Change::RemoveClient(fd) => {
				lock.retain(|&(x, _)| x != fd);
				clients.remove(&fd);

				let mut lock = POINTER_OVER.lock().unwrap();

				if let Some(value) = &mut *lock {
					if value.fd == fd {
						*lock = None;
					}
				}

				false
			}
			Change::Pick(fd, toplevel) => {
				let size_before = lock.len();
				lock.retain(|&x| x != (fd, toplevel));
				assert!(lock.len() == (size_before - 1));

				lock.push_front((fd, toplevel));
				true
			}
		};

		if i == 0 {
			should_leave_from_old = x;
		}
	}

	let current = lock.iter().next().cloned();

	if old == current && !should_recompute_size_and_pos {
		return Ok(());
	}

	const GAP: i32 = 0;
	const WIDTH: i32 = 2560;
	const HEIGHT: i32 = 1440;

	let get_pos_and_size = |index: u32, amount: u32| -> (Point, Point) {
		match amount {
			0 => {
				unreachable!();
			}
			1 => (
				Point(0 + GAP, 0 + GAP),
				Point(WIDTH - GAP * 2, HEIGHT - GAP * 2),
			),
			2.. => match index {
				0 => (
					Point(0 + GAP, 0 + GAP),
					Point(WIDTH / 2 - GAP * 2, HEIGHT - GAP * 2),
				),
				1.. => {
					let frac = ((1. / (amount - 1) as f32) * HEIGHT as f32) as i32;
					(
						Point(WIDTH / 2 + GAP, frac * (index as i32 - 1) + GAP),
						Point(WIDTH / 2 - GAP * 2, frac - GAP * 2),
					)
				}
			},
		}
	};

	let mut index = 0;

	for client in clients.values_mut() {
		for xdg_toplevel in client.objects_mut::<wl::XdgToplevel>() {
			let (pos, size) = get_pos_and_size(index as _, lock.len() as _);

			xdg_toplevel.position = pos;
			xdg_toplevel.size = Some(size);

			if Some((client.fd, xdg_toplevel.object_id)) != old
				&& Some((client.fd, xdg_toplevel.object_id)) != current
			{
				xdg_toplevel.configure(client)?;
			}

			index += 1;
		}
	}

	if should_leave_from_old {
		if let Some((fd, id)) = old {
			let client = clients.get_mut(&fd).unwrap();

			let xdg_toplevel = client.get_object_mut(id).unwrap();
			let xdg_surface = client.get_object(xdg_toplevel.surface)?;
			let surface = client.get_object(xdg_surface.surface)?;

			for keyboard in client.objects_mut::<wl::Keyboard>() {
				keyboard.leave(client, surface.object_id)?;
			}

			xdg_toplevel.remove_state(4);
			xdg_toplevel.configure(client)?;
		}
	}

	if let Some((fd, id)) = current {
		let client = clients.get_mut(&fd).unwrap();

		let xdg_toplevel = client.get_object_mut(id).unwrap();
		let xdg_surface = client.get_object(xdg_toplevel.surface)?;
		let surface = client.get_object(xdg_surface.surface)?;

		for keyboard in client.objects_mut::<wl::Keyboard>() {
			keyboard.enter(client, surface.object_id)?;
		}

		xdg_toplevel.add_state(4);
		xdg_toplevel.configure(client)?;
	}

	Ok(())
}

pub fn on_cursor_move(cursor_position: (i32, i32)) -> Result<()> {
	let mut clients = CLIENTS.lock().unwrap();
	let cursor_position = Point(cursor_position.0, cursor_position.1);

	*POINTER_POSITION.lock().unwrap() = cursor_position;

	for client in clients.values_mut() {
		for seat in client.objects_mut::<wl::Seat>() {
			seat.pointer_position = cursor_position;
		}
	}

	let old = {
		let mut lock = POINTER_OVER.lock().unwrap();
		let ret = *lock;

		*lock = None;
		ret
	};

	let mut moving = None;

	'outer: for (client, window) in WINDOW_STACK.lock().unwrap().iter() {
		if POINTER_OVER.lock().unwrap().is_some() {
			break;
		}

		let client = clients.get_mut(client).unwrap();

		for seat in client.objects_mut::<wl::Seat>() {
			if seat.moving_toplevel.is_some() {
				moving = Some((client.fd, seat.object_id));
				break;
			}
		}

		if moving.is_some() {
			break;
		}

		fn is_point_inside_area(cursor: Point, position: Point, size: Point) -> bool {
			cursor.0 > position.0
				&& cursor.1 > position.1
				&& cursor.0 <= position.0 + size.0
				&& cursor.1 <= position.1 + size.1
		}

		fn is_cursor_over_surface(
			cursor_position: Point,
			surface_position: Point,
			surface: &wl::Surface,
		) -> bool {
			if let Some(input_region) = &surface.current_input_region {
				for area in &input_region.areas {
					let position = surface_position + area.0;

					if is_point_inside_area(cursor_position, position, area.1) {
						return true;
					}
				}
			} else if let Some(&(size, ..)) = surface.data.as_ref() {
				return is_point_inside_area(cursor_position, surface_position, size);
			}

			false
		}

		fn do_stuff(
			client: &mut wl::Client,
			toplevel: &wl::XdgToplevel,
			surface: &wl::Surface,
			cursor_position: Point,
			surface_position: Point,
		) {
			if is_cursor_over_surface(cursor_position, surface_position, surface) {
				*POINTER_OVER.lock().unwrap() = Some(PointerOver {
					fd: client.fd,
					toplevel: toplevel.object_id,
					surface: surface.object_id,
					position: cursor_position - surface_position,
				});
			}

			for child in &surface.children {
				let sub_surface = client.get_object(*child).unwrap();
				let surface = client.get_object(sub_surface.surface).unwrap();

				do_stuff(
					client,
					toplevel,
					surface,
					cursor_position,
					surface_position + sub_surface.position,
				);
			}
		}

		let toplevel = client.get_object(*window).unwrap();
		let xdg_surface = client.get_object(toplevel.surface).unwrap();
		let surface = client.get_object(xdg_surface.surface).unwrap();

		let position = toplevel.position - xdg_surface.position;

		for &popup in &xdg_surface.popups {
			let popup = client.get_object(popup).unwrap();
			let xdg_surface = client.get_object(popup.xdg_surface).unwrap();
			let surface = client.get_object(xdg_surface.surface).unwrap();

			let position = (position - xdg_surface.position) + popup.position;

			do_stuff(client, toplevel, surface, cursor_position, position);

			if POINTER_OVER.lock().unwrap().is_some() {
				break 'outer;
			}
		}

		do_stuff(client, toplevel, surface, cursor_position, position);
	}

	if let Some((fd, seat)) = moving {
		let client = clients.get_mut(&fd).unwrap();
		let seat = client.get_object_mut(seat).unwrap();

		if let Some((toplevel, window_start_pos, pointer_start_pos)) = &seat.moving_toplevel {
			let toplevel = client.get_object_mut(*toplevel).unwrap();

			toplevel.position = *window_start_pos + (seat.pointer_position - *pointer_start_pos);
		}
	}

	let current = *POINTER_OVER.lock().unwrap();

	if old.is_none() && current.is_none() {
		return Ok(());
	}

	if old.map(|x| (x.fd, x.surface)) != current.map(|x| (x.fd, x.surface)) {
		if let Some(PointerOver { fd, surface, .. }) = old {
			let client = clients.get_mut(&fd).unwrap();

			for pointer in client.objects_mut::<wl::Pointer>() {
				pointer.leave(client, surface).unwrap();
				pointer.frame(client).unwrap();
			}
		}

		if let Some(PointerOver {
			fd,
			surface,
			position,
			..
		}) = current
		{
			let client = clients.get_mut(&fd).unwrap();

			for pointer in client.objects_mut::<wl::Pointer>() {
				pointer.enter(client, surface, position).unwrap();
				pointer.frame(client).unwrap();
			}
		}
	} else if old.map(|x| x.position) != current.map(|x| x.position) {
		let PointerOver { fd, position, .. } = current.unwrap();

		let client = clients.get_mut(&fd).unwrap();

		for pointer in client.objects_mut::<wl::Pointer>() {
			pointer.motion(client, position).unwrap();
			pointer.frame(client).unwrap();
		}
	}

	Ok(())
}

pub fn on_mouse_button(button: u32, input_state: u32) -> Result<()> {
	let mut clients = CLIENTS.lock().unwrap();

	for client in clients.values_mut() {
		for seat in client.objects_mut::<wl::Seat>() {
			if seat.moving_toplevel.is_some() {
				assert!(input_state == 0);
				seat.moving_toplevel = None;

				return Ok(());
			}
		}
	}

	if let Some(PointerOver { fd, toplevel, .. }) = *POINTER_OVER.lock().unwrap() {
		let client = clients.get_mut(&fd).unwrap();

		for pointer in client.objects_mut::<wl::Pointer>() {
			pointer.button(client, button, input_state).unwrap();
			pointer.frame(client).unwrap();
		}

		let Some(&topmost) = WINDOW_STACK.lock().unwrap().front() else {
			panic!();
		};

		if topmost != (fd, toplevel) {
			CHANGES.lock().unwrap().push(Change::Pick(fd, toplevel));
		}
	}
	Ok(())
}

pub fn on_mouse_scroll(value: f64, discrete: i32, axis: u32) -> Result<()> {
	let mut clients = CLIENTS.lock().unwrap();

	if let Some(PointerOver { fd, .. }) = *POINTER_OVER.lock().unwrap() {
		let client = clients.get_mut(&fd).unwrap();

		for pointer in client.objects_mut::<wl::Pointer>() {
			pointer.axis_source(client, 0)?;
			pointer.axis_discrete(client, axis, discrete)?;
			pointer.axis(client, axis, value)?;
			pointer.frame(client)?;
		}
	}

	Ok(())
}

pub fn on_keyboard_button(code: u32, input_state: u32) -> Result<()> {
	let mut clients = CLIENTS.lock().unwrap();
	let xkb_state_lock = XKB_STATE.lock().unwrap();
	let Some(xkb_state) = &*xkb_state_lock else {
		panic!();
	};

	xkb_state.state.update_key(code + 8, input_state as _);
	let depressed = xkb_state.state.serialize_mods(1);

	if (depressed & 64) != 0 {
		if code == 1 && input_state == 1 {
			set_quit();
			return Ok(());
		}

		if code == 20 && input_state == 1 {
			std::process::Command::new("foot")
				.env("WAYLAND_DISPLAY", "wayland-1")
				.stdout(std::process::Stdio::null())
				.stderr(std::process::Stdio::null())
				.spawn()
				.unwrap();

			return Ok(());
		}

		if code == 46 && input_state == 1 {
			if let Some((fd, xdg_toplevel)) = WINDOW_STACK.lock().unwrap().front() {
				let client = clients.get_mut(fd).unwrap();
				let xdg_toplevel = client.get_object(*xdg_toplevel)?;

				xdg_toplevel.close(client)?;
			}

			return Ok(());
		}
	}

	if let Some((client, _window)) = WINDOW_STACK.lock().unwrap().iter().next() {
		let client = clients.get_mut(client).unwrap();

		for keyboard in client.objects_mut::<wl::Keyboard>() {
			if keyboard.key_states[code as usize] != (input_state != 0) {
				keyboard.key_states[code as usize] = input_state != 0;
				keyboard.key(client, code, input_state).unwrap();
			}

			keyboard.modifiers(client, depressed).unwrap();
		}
	}
	Ok(())
}

static QUIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn quit() -> bool {
	QUIT.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn set_quit() {
	QUIT.store(true, std::sync::atomic::Ordering::Relaxed);
}
