use crate::{wl, Result};

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
	pub position: (i32, i32),
}

pub static POINTER_OVER: std::sync::LazyLock<std::sync::Mutex<Option<PointerOver>>> =
	std::sync::LazyLock::new(Default::default);

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
