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
	for change in std::mem::take(&mut *changes()) {
		let mut lock = window_stack();
		let old = lock.iter().next().cloned();

		let should_leave_from_old = match change {
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

		let current = lock.iter().next().cloned();

		if old == current {
			continue;
		}

		if should_leave_from_old {
			if let Some((fd, id)) = old {
				let client = clients.get_mut(&fd).unwrap();

				let xdg_toplevel = client.get_object(id)?;
				let xdg_surface = client.get_object(xdg_toplevel.surface)?;
				let surface = client.get_object(xdg_surface.surface).unwrap();

				for keyboard in client.objects_mut::<wl::Keyboard>() {
					keyboard.leave(client, surface.object_id)?;
				}

				xdg_toplevel.configure(client, 0, 0, &[])?;
			}
		}

		if let Some((fd, id)) = current {
			let client = clients.get_mut(&fd).unwrap();

			let xdg_toplevel = client.get_object(id)?;
			let xdg_surface = client.get_object(xdg_toplevel.surface)?;
			let surface = client.get_object(xdg_surface.surface)?;

			for keyboard in client.objects_mut::<wl::Keyboard>() {
				keyboard.enter(client, surface.object_id)?;
			}

			xdg_toplevel.configure(client, 0, 0, &[4])?;
		}
	}

	Ok(())
}

pub fn add_change(change: Change) {
	changes().push(change);
}
