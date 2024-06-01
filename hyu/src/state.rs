use crate::{wl, Result};

static CLIENTS: std::sync::OnceLock<
	std::sync::Mutex<std::collections::HashMap<std::os::fd::RawFd, wl::Client>>,
> = std::sync::OnceLock::new();

pub fn clients<'a>(
) -> std::sync::MutexGuard<'static, std::collections::HashMap<std::os::fd::RawFd, wl::Client<'a>>> {
	CLIENTS
		.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
		.lock()
		.unwrap()
}

static WINDOW_STACK: std::sync::OnceLock<
	std::sync::Mutex<std::collections::VecDeque<(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>)>>,
> = std::sync::OnceLock::new();

pub fn window_stack() -> std::sync::MutexGuard<
	'static,
	std::collections::VecDeque<(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>)>,
> {
	WINDOW_STACK.get_or_init(Default::default).lock().unwrap()
}

pub enum Change {
	Push(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
	RemoveToplevel(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
	RemoveSurface(std::os::fd::RawFd, wl::Id<wl::Surface>),
	RemoveClient(std::os::fd::RawFd),
	Pick(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
}

static CHANGES: std::sync::OnceLock<std::sync::Mutex<Vec<Change>>> = std::sync::OnceLock::new();

pub fn changes() -> std::sync::MutexGuard<'static, Vec<Change>> {
	CHANGES.get_or_init(Default::default).lock().unwrap()
}

#[derive(Clone, Copy)]
pub struct PointerOver {
	pub fd: std::os::fd::RawFd,
	pub toplevel: wl::Id<wl::XdgToplevel>,
	pub surface: wl::Id<wl::Surface>,
	pub position: (i32, i32),
}

static POINTER_OVER: std::sync::OnceLock<std::sync::Mutex<Option<PointerOver>>> =
	std::sync::OnceLock::new();

pub fn pointer_over() -> std::sync::MutexGuard<'static, Option<PointerOver>> {
	POINTER_OVER.get_or_init(Default::default).lock().unwrap()
}

pub fn process_focus_changes(
	clients: &mut std::sync::MutexGuard<
		'_,
		std::collections::HashMap<std::os::fd::RawFd, wl::Client>,
	>,
) -> Result<()> {
	let mut lock = window_stack();
	let old = lock.iter().next().cloned();

	let mut should_leave_from_old = false;
	let mut should_recompute_size_and_pos = false;

	for (i, change) in std::mem::take(&mut *changes()).into_iter().enumerate() {
		should_recompute_size_and_pos = true;

		let x = match change {
			Change::Push(fd, id) => {
				lock.push_front((fd, id));
				true
			}
			Change::RemoveToplevel(fd, id) => {
				lock.retain(|&x| x != (fd, id));

				let mut lock = pointer_over();

				if let Some(value) = &mut *lock {
					if value.fd == fd && value.toplevel == id {
						*lock = None;
					}
				}

				false
			}
			Change::RemoveSurface(fd, id) => {
				let mut lock = pointer_over();

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

				let mut lock = pointer_over();

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

	let get_pos_and_size = |index: u32, amount: u32| -> ((u32, u32), (u32, u32)) {
		match amount {
			0 => {
				unreachable!();
			}
			1 => ((0, 0), (1280, 720)),
			2.. => match index {
				0 => ((0, 0), (1280 / 2, 720)),
				1.. => {
					let frac = ((1. / (amount - 1) as f32) * 720.0) as u32;
					((1280 / 2, frac * (index - 1)), (1280 / 2, frac))
				}
			},
		}
	};

	for (index, client) in clients.values_mut().enumerate() {
		for xdg_toplevel in client.objects_mut::<wl::XdgToplevel>() {
			let (pos, size) = get_pos_and_size(index as _, lock.len() as _);
			xdg_toplevel.configure(client, size.0 as _, size.1 as _, &[1])?;
			xdg_toplevel.position = (pos.0 as _, pos.1 as _);
			xdg_toplevel.size = Some((size.0 as _, size.1 as _));
		}
	}

	if should_leave_from_old {
		if let Some((fd, id)) = old {
			let client = clients.get_mut(&fd).unwrap();

			let xdg_toplevel = client.get_object(id).unwrap();
			let xdg_surface = client.get_object(xdg_toplevel.surface)?;
			let surface = client.get_object(xdg_surface.surface).unwrap();

			for keyboard in client.objects_mut::<wl::Keyboard>() {
				keyboard.leave(client, surface.object_id)?;
			}

			let (w, h) = xdg_toplevel.size.unwrap_or((0, 0));
			xdg_toplevel.configure(client, w, h, &[1])?;
		}
	}

	if let Some((fd, id)) = current {
		let client = clients.get_mut(&fd).unwrap();

		let xdg_toplevel = client.get_object(id).unwrap();
		let xdg_surface = client.get_object(xdg_toplevel.surface)?;
		let surface = client.get_object(xdg_surface.surface)?;

		for keyboard in client.objects_mut::<wl::Keyboard>() {
			keyboard.enter(client, surface.object_id)?;
		}

		let (w, h) = xdg_toplevel.size.unwrap_or((0, 0));
		xdg_toplevel.configure(client, w, h, &[1, 4])?;
	}

	Ok(())
}

pub fn add_change(change: Change) {
	changes().push(change);
}
