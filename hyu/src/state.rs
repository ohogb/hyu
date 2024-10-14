use std::{
	io::{Seek as _, Write as _},
	os::fd::{FromRawFd as _, IntoRawFd as _},
};

use color_eyre::eyre::OptionExt as _;

use crate::{wl, xkb, Client, Point, Result};

pub enum Change {
	Push(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
	RemoveToplevel(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
	RemoveSurface(std::os::fd::RawFd, wl::Id<wl::Surface>),
	RemoveClient(std::os::fd::RawFd),
	Pick(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
	MoveDown(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
	MoveUp(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
}

#[derive(Clone, Copy)]
pub struct PointerOver {
	pub fd: std::os::fd::RawFd,
	pub toplevel: wl::Id<wl::XdgToplevel>,
	pub surface: wl::Id<wl::Surface>,
	pub position: Point,
}

pub struct XkbState {
	pub context: xkb::Context,
	pub keymap: xkb::Keymap,
	pub state: xkb::State,
	pub keymap_file: (std::os::fd::RawFd, u64),
}

pub struct State {
	pub drm: crate::backend::drm::State,
	pub input: crate::backend::input::State,
	pub compositor: CompositorState,
}

pub struct CompositorState {
	pub clients: std::collections::HashMap<std::os::fd::RawFd, Client>,
	pub windows: Vec<std::rc::Rc<(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>)>>,
	pub focused_window: Option<std::rc::Weak<(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>)>>,
	pub changes: Vec<Change>,
	pub pointer_over: Option<PointerOver>,
	pub pointer_position: Point,
	pub xkb_state: XkbState,
	pub width: u16,
	pub height: u16,
}

impl CompositorState {
	pub fn create(width: u16, height: u16, layout: impl AsRef<str>) -> Result<Self> {
		let xkb_context = xkb::Context::create().ok_or_eyre("failed to create xkb context")?;

		let xkb_keymap =
			xkb::Keymap::create(&xkb_context, layout).ok_or_eyre("failed to create xkb keymap")?;

		let xkb_state = xkb::State::new(&xkb_keymap).ok_or_eyre("failed to create xkb state")?;

		let (fd, path) = nix::unistd::mkstemp("/tmp/temp_XXXXXX")?;
		nix::unistd::unlink(&path)?;

		let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
		write!(file, "{}", xkb_keymap.get_as_string())?;

		let size = file.stream_len()?;
		let fd = file.into_raw_fd();

		Ok(Self {
			clients: Default::default(),
			windows: Default::default(),
			focused_window: Default::default(),
			changes: Default::default(),
			pointer_over: Default::default(),
			pointer_position: Default::default(),
			xkb_state: XkbState {
				context: xkb_context,
				keymap: xkb_keymap,
				state: xkb_state,
				keymap_file: (fd, size),
			},
			width,
			height,
		})
	}

	pub fn process_focus_changes(&mut self) -> Result<()> {
		let old = self.get_focused_window();

		let mut should_leave_from_old = false;

		let changes = std::mem::take(&mut self.changes);
		let should_recompute_size_and_pos = !changes.is_empty();

		for (i, change) in changes.into_iter().enumerate() {
			let x = match change {
				Change::Push(fd, id) => {
					let rc = std::rc::Rc::new((fd, id));

					self.windows.insert(0, rc.clone());
					self.focused_window = Some(std::rc::Rc::downgrade(&rc));

					let client = self.clients.get_mut(&fd).unwrap();

					let toplevel = client.get_object_mut(id)?;
					toplevel.add_state(1);

					true
				}
				Change::RemoveToplevel(fd, id) => {
					self.windows.retain(|x| **x != (fd, id));

					if self.get_focused_window().is_none() {
						self.focused_window = self.windows.first().map(std::rc::Rc::downgrade);
					}

					if let Some(value) = &self.pointer_over {
						if value.fd == fd && value.toplevel == id {
							self.pointer_over = None;
						}
					}

					false
				}
				Change::RemoveSurface(fd, id) => {
					if let Some(value) = &self.pointer_over {
						if value.fd == fd && value.surface == id {
							self.pointer_over = None;
						}
					}

					false
				}
				Change::RemoveClient(fd) => {
					self.windows.retain(|x| x.0 != fd);
					self.clients.remove(&fd);

					if self.get_focused_window().is_none() {
						self.focused_window = self.windows.first().map(std::rc::Rc::downgrade);
					}

					if let Some(value) = &self.pointer_over {
						if value.fd == fd {
							self.pointer_over = None;
						}
					}

					false
				}
				Change::Pick(fd, toplevel) => {
					self.focused_window = self
						.windows
						.iter()
						.find(|x| ***x == (fd, toplevel))
						.map(std::rc::Rc::downgrade);

					assert!(self.focused_window.is_some());
					true
				}
				Change::MoveDown(fd, xdg_toplevel) => {
					let Some(index) = self
						.windows
						.iter()
						.enumerate()
						.find(|(_, x)| ***x == (fd, xdg_toplevel))
						.map(|(x, _)| x)
					else {
						continue;
					};

					if index >= (self.windows.len() - 1) {
						continue;
					};

					self.windows.swap(index, index + 1);
					false
				}
				Change::MoveUp(fd, xdg_toplevel) => {
					let Some(index) = self
						.windows
						.iter()
						.enumerate()
						.find(|(_, x)| ***x == (fd, xdg_toplevel))
						.map(|(x, _)| x)
					else {
						continue;
					};

					if index == 0 {
						continue;
					};

					self.windows.swap(index, index - 1);
					false
				}
			};

			if i == 0 {
				should_leave_from_old = x;
			}
		}

		let current = self.get_focused_window();

		if !should_recompute_size_and_pos {
			return Ok(());
		}

		const GAP: i32 = 0;

		let width = self.width as i32;
		let height = self.height as i32;

		let get_pos_and_size = |index: u32, amount: u32| -> (Point, Point) {
			match amount {
				0 => {
					unreachable!();
				}
				1 => (
					Point(0 + GAP, 0 + GAP),
					Point(width - GAP * 2, height - GAP * 2),
				),
				2.. => match index {
					0 => (
						Point(0 + GAP, 0 + GAP),
						Point(width / 2 - GAP * 2, height - GAP * 2),
					),
					1.. => {
						let frac = ((1. / (amount - 1) as f32) * height as f32) as i32;
						(
							Point(width / 2 + GAP, frac * (index as i32 - 1) + GAP),
							Point(width / 2 - GAP * 2, frac - GAP * 2),
						)
					}
				},
			}
		};

		let mut leave = None;
		let mut enter = None;

		for (index, (fd, xdg_toplevel)) in self.windows.iter().map(|x| **x).enumerate() {
			let client = self.clients.get_mut(&fd).unwrap();
			let xdg_toplevel = client.get_object_mut(xdg_toplevel)?;

			let (pos, size) = get_pos_and_size(index as _, self.windows.len() as _);

			xdg_toplevel.position = pos;
			xdg_toplevel.size = Some(size);

			let xdg_surface = client.get_object(xdg_toplevel.surface)?;
			let surface = client.get_object(xdg_surface.surface)?;

			if should_leave_from_old && old == Some((fd, xdg_toplevel.object_id)) {
				leave = Some((fd, xdg_toplevel.object_id, surface.object_id));
				continue;
			} else if current == Some((fd, xdg_toplevel.object_id)) {
				enter = Some((fd, xdg_toplevel.object_id, surface.object_id));
				continue;
			}

			xdg_toplevel.configure(client)?;
		}

		if let Some((fd, xdg_toplevel, surface)) = leave {
			let client = self.clients.get_mut(&fd).unwrap();
			let xdg_toplevel = client.get_object_mut(xdg_toplevel)?;

			for keyboard in client.objects_mut::<wl::Keyboard>() {
				keyboard.leave(client, surface)?;
			}

			xdg_toplevel.remove_state(4);
			xdg_toplevel.configure(client)?;
		}

		if let Some((fd, xdg_toplevel, surface)) = enter {
			let client = self.clients.get_mut(&fd).unwrap();
			let xdg_toplevel = client.get_object_mut(xdg_toplevel)?;

			let depressed = self.xkb_state.state.serialize_mods(1);

			for keyboard in client.objects_mut::<wl::Keyboard>() {
				keyboard.enter(client, surface)?;
				keyboard.modifiers(client, depressed)?;
			}

			xdg_toplevel.add_state(4);
			xdg_toplevel.configure(client)?;
		}

		self.calculate_pointer_focus()
	}

	fn calculate_pointer_focus(&mut self) -> Result<()> {
		let old = self.pointer_over;
		let mut new = None;

		'outer: for (fd, xdg_toplevel) in self.windows.iter().map(|x| **x) {
			let client = self.clients.get_mut(&fd).unwrap();

			fn is_cursor_over_surface(
				cursor_position: Point,
				surface_position: Point,
				surface: &wl::Surface,
			) -> bool {
				if let Some(input_region) = &surface.current.input_region {
					for area in &input_region.areas {
						let position = surface_position + area.0;

						if cursor_position.is_inside((position, area.1)) {
							return true;
						}
					}
				} else if let Some(&(size, ..)) = surface.data.as_ref() {
					return cursor_position.is_inside((surface_position, size));
				}

				false
			}

			fn recurse(
				pointer_over: &mut Option<PointerOver>,
				client: &mut Client,
				toplevel: &wl::XdgToplevel,
				surface: &wl::Surface,
				cursor_position: Point,
				surface_position: Point,
			) -> Result<()> {
				if is_cursor_over_surface(cursor_position, surface_position, surface) {
					*pointer_over = Some(PointerOver {
						fd: client.fd,
						toplevel: toplevel.object_id,
						surface: surface.object_id,
						position: cursor_position - surface_position,
					});
				}

				for child in &surface.children {
					let sub_surface = client.get_object(*child).unwrap();
					let surface = client.get_object(sub_surface.surface)?;

					recurse(
						pointer_over,
						client,
						toplevel,
						surface,
						cursor_position,
						surface_position + sub_surface.position,
					)?;
				}

				Ok(())
			}

			let toplevel = client.get_object(xdg_toplevel)?;
			let xdg_surface = client.get_object(toplevel.surface)?;
			let surface = client.get_object(xdg_surface.surface)?;

			let position = toplevel.position - xdg_surface.position;

			for &popup in &xdg_surface.popups {
				let popup = client.get_object(popup)?;
				let xdg_surface = client.get_object(popup.xdg_surface)?;
				let surface = client.get_object(xdg_surface.surface)?;

				let position = (position - xdg_surface.position) + popup.position;

				recurse(
					&mut new,
					client,
					toplevel,
					surface,
					self.pointer_position,
					position,
				)?;

				if new.is_some() {
					break 'outer;
				}
			}

			recurse(
				&mut new,
				client,
				toplevel,
				surface,
				self.pointer_position,
				position,
			)?;

			if new.is_some() {
				break;
			}
		}

		if old.map(|x| (x.fd, x.surface)) != new.map(|x| (x.fd, x.surface)) {
			if let Some(PointerOver { fd, surface, .. }) = old {
				let client = self.clients.get_mut(&fd).unwrap();
				let mut pointers = client.objects_mut::<wl::Pointer>();

				let display = client.get_object_mut(wl::Id::<wl::Display>::new(1))?;
				let serial = display.new_serial();

				for pointer in &mut pointers {
					pointer.leave(client, serial, surface)?;
				}

				for pointer in &mut pointers {
					pointer.frame(client)?;
				}
			}

			if let Some(PointerOver {
				fd,
				surface,
				position,
				..
			}) = new
			{
				let client = self.clients.get_mut(&fd).unwrap();
				let mut pointers = client.objects_mut::<wl::Pointer>();

				let display = client.get_object_mut(wl::Id::<wl::Display>::new(1))?;
				let serial = display.new_serial();

				for pointer in &mut pointers {
					pointer.enter(client, serial, surface, position)?;
				}

				for pointer in &mut pointers {
					pointer.frame(client)?;
				}
			}
		} else if old.map(|x| x.position) != new.map(|x| x.position) {
			let PointerOver { fd, position, .. } = new.unwrap();

			let client = self.clients.get_mut(&fd).unwrap();
			let mut pointers = client.objects_mut::<wl::Pointer>();

			for pointer in &mut pointers {
				pointer.motion(client, position)?;
			}

			for pointer in &mut pointers {
				pointer.frame(client)?;
			}
		}

		self.pointer_over = new;
		Ok(())
	}

	pub fn on_cursor_move(&mut self, cursor_position: (i32, i32)) -> Result<()> {
		let cursor_position = Point(cursor_position.0, cursor_position.1);

		self.pointer_position = cursor_position;
		self.calculate_pointer_focus()
	}

	pub fn on_mouse_button(&mut self, button: u32, input_state: u32) -> Result<()> {
		if let Some(PointerOver { fd, toplevel, .. }) = self.pointer_over {
			let client = self.clients.get_mut(&fd).unwrap();

			let display = client.get_object_mut(wl::Id::<wl::Display>::new(1))?;
			let serial = display.new_serial();

			for pointer in client.objects_mut::<wl::Pointer>() {
				pointer.button(client, serial, button, input_state)?;
			}

			for pointer in client.objects_mut::<wl::Pointer>() {
				pointer.frame(client)?;
			}

			let Some(focused_window) = self.get_focused_window() else {
				panic!();
			};

			if focused_window != (fd, toplevel) {
				self.changes.push(Change::Pick(fd, toplevel));
			}

			self.process_focus_changes()?;
		}

		Ok(())
	}

	pub fn on_mouse_scroll(&mut self, value: f64, discrete: i32, axis: u32) -> Result<()> {
		if let Some(PointerOver { fd, .. }) = self.pointer_over {
			let client = self.clients.get_mut(&fd).unwrap();

			for pointer in client.objects_mut::<wl::Pointer>() {
				pointer.axis_source(client, 0)?;
				pointer.axis_discrete(client, axis, discrete)?;
				pointer.axis(client, axis, value)?;
			}

			for pointer in client.objects_mut::<wl::Pointer>() {
				pointer.frame(client)?;
			}
		}

		Ok(())
	}

	pub fn on_keyboard_button(&mut self, code: u32, input_state: u32) -> Result<()> {
		self.xkb_state.state.update_key(code + 8, input_state as _);
		let depressed = self.xkb_state.state.serialize_mods(1);

		if (depressed & 64) != 0 {
			if code == 1 && input_state == 1 {
				color_eyre::eyre::bail!("quit");
				// return Ok(());
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
				if let Some((fd, xdg_toplevel)) = self.get_focused_window() {
					let client = self.clients.get_mut(&fd).unwrap();
					let xdg_toplevel = client.get_object(xdg_toplevel)?;

					xdg_toplevel.close(client)?;
				}

				return Ok(());
			}

			if (depressed & 1) != 0 {
				if code == 36 && input_state == 1 {
					let Some((fd, xdg_toplevel)) = self.get_focused_window() else {
						return Ok(());
					};

					self.changes.push(Change::MoveDown(fd, xdg_toplevel));
					self.process_focus_changes()?;

					return Ok(());
				}

				if code == 37 && input_state == 1 {
					let Some((fd, xdg_toplevel)) = self.get_focused_window() else {
						return Ok(());
					};

					self.changes.push(Change::MoveUp(fd, xdg_toplevel));
					self.process_focus_changes()?;

					return Ok(());
				}
			}

			if code == 36 && input_state == 1 {
				let Some((fd, xdg_toplevel)) = self.get_focused_window() else {
					return Ok(());
				};

				let Some(index) = self
					.windows
					.iter()
					.enumerate()
					.find(|(_, x)| ***x == (fd, xdg_toplevel))
					.map(|(x, _)| x)
				else {
					return Ok(());
				};

				if index >= (self.windows.len() - 1) {
					return Ok(());
				};

				let Some((fd, xdg_toplevel)) = self.windows.get(index + 1).map(|x| **x) else {
					panic!();
				};

				self.changes.push(Change::Pick(fd, xdg_toplevel));
				self.process_focus_changes()?;

				return Ok(());
			}

			if code == 37 && input_state == 1 {
				let Some((fd, xdg_toplevel)) = self.get_focused_window() else {
					return Ok(());
				};

				let Some(index) = self
					.windows
					.iter()
					.enumerate()
					.find(|(_, x)| ***x == (fd, xdg_toplevel))
					.map(|(x, _)| x)
				else {
					return Ok(());
				};

				if index == 0 {
					return Ok(());
				};

				let Some((fd, xdg_toplevel)) = self.windows.get(index - 1).map(|x| **x) else {
					panic!();
				};

				self.changes.push(Change::Pick(fd, xdg_toplevel));
				self.process_focus_changes()?;

				return Ok(());
			}

			return Ok(());
		}

		if let Some((fd, _)) = self.get_focused_window() {
			let client = self.clients.get_mut(&fd).unwrap();

			for keyboard in client.objects_mut::<wl::Keyboard>() {
				keyboard.key(client, code, input_state)?;
				keyboard.modifiers(client, depressed)?;
			}
		}

		Ok(())
	}

	pub fn get_focused_window(&self) -> Option<(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>)> {
		self.focused_window
			.as_ref()
			.and_then(|x| x.upgrade())
			.map(|x| *x)
	}

	pub fn render(&mut self, gl: &mut crate::backend::gl::Renderer) -> Result<()> {
		for (fd, xdg_toplevel) in self.windows.iter().map(|x| **x) {
			let client = self.clients.get_mut(&fd).unwrap();

			let mut draw = |client: &mut Client,
			                toplevel_position: Point,
			                xdg_surface: &wl::XdgSurface,
			                surface: &mut wl::Surface|
			 -> Result<()> {
				for (position, size, surface_id) in surface.get_front_buffers(client)? {
					let surface = client.get_object(surface_id)?;

					let Some((.., wl::SurfaceTexture::Gl(texture))) = &surface.data else {
						panic!();
					};

					gl.quad(
						toplevel_position - xdg_surface.position + position,
						size,
						texture,
					);
				}

				Ok(())
			};

			let toplevel = client.get_object(xdg_toplevel)?;

			let xdg_surface = client.get_object(toplevel.surface)?;
			let surface = client.get_object_mut(xdg_surface.surface)?;

			draw(client, toplevel.position, xdg_surface, surface)?;

			for &popup in &xdg_surface.popups {
				let popup = client.get_object(popup)?;

				let xdg_surface = client.get_object(popup.xdg_surface)?;
				let surface = client.get_object_mut(xdg_surface.surface)?;

				let position = toplevel.position + popup.position;

				draw(client, position, xdg_surface, surface)?;
			}
		}

		let should_hide_cursor = if let Some(a) = &self.pointer_over {
			let client = self.clients.get(&a.fd).unwrap();
			client
				.objects_mut::<wl::Pointer>()
				.iter()
				.fold(false, |acc, x| acc | x.should_hide_cursor)
		} else {
			false
		};

		if !should_hide_cursor {
			let cursor_pos = self.pointer_position;
			gl.quad(cursor_pos, Point(2, 2), &gl.cursor_texture.clone());
		}

		Ok(())
	}
}
