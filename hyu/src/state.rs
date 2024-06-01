use crate::{wl, Point, Result};

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

	const GAP: i32 = 10;

	let get_pos_and_size = |index: u32, amount: u32| -> (Point, Point) {
		match amount {
			0 => {
				unreachable!();
			}
			1 => (
				Point(0 + GAP, 0 + GAP),
				Point(1280 - GAP * 2, 720 - GAP * 2),
			),
			2.. => match index {
				0 => (
					Point(0 + GAP, 0 + GAP),
					Point(1280 / 2 - GAP * 2, 720 - GAP * 2),
				),
				1.. => {
					let frac = ((1. / (amount - 1) as f32) * 720.0) as i32;
					(
						Point(1280 / 2 + GAP, frac * (index as i32 - 1) + GAP),
						Point(1280 / 2 - GAP * 2, frac - GAP * 2),
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
				xdg_toplevel.configure(client, size, &[1])?;
			}

			index += 1;
		}
	}

	if should_leave_from_old {
		if let Some((fd, id)) = old {
			let client = clients.get_mut(&fd).unwrap();

			let xdg_toplevel = client.get_object(id).unwrap();
			let xdg_surface = client.get_object(xdg_toplevel.surface)?;
			let surface = client.get_object(xdg_surface.surface)?;

			for keyboard in client.objects_mut::<wl::Keyboard>() {
				keyboard.leave(client, surface.object_id)?;
			}

			let size = xdg_toplevel.size.unwrap_or(Point(0, 0));
			xdg_toplevel.configure(client, size, &[1])?;
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

		let size = xdg_toplevel.size.unwrap_or(Point(0, 0));
		xdg_toplevel.configure(client, size, &[1, 4])?;
	}

	Ok(())
}
