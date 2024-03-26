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
	Remove(std::os::fd::RawFd, wl::Id<wl::XdgToplevel>),
	RemoveAll(std::os::fd::RawFd),
}

static CHANGES: std::sync::OnceLock<std::sync::Mutex<Vec<Change>>> = std::sync::OnceLock::new();

pub fn changes() -> std::sync::MutexGuard<'static, Vec<Change>> {
	CHANGES.get_or_init(Default::default).lock().unwrap()
}

static POINTER_OVER: std::sync::OnceLock<
	std::sync::Mutex<Option<(std::os::fd::RawFd, wl::Id<wl::Surface>, (i32, i32))>>,
> = std::sync::OnceLock::new();

pub fn pointer_over(
) -> std::sync::MutexGuard<'static, Option<(std::os::fd::RawFd, wl::Id<wl::Surface>, (i32, i32))>> {
	POINTER_OVER.get_or_init(Default::default).lock().unwrap()
}

pub fn process_focus_changes() -> Result<bool> {
	let mut has_pending_messages = false;

	for change in std::mem::take(&mut *changes()) {
		let mut lock = window_stack();
		let old = lock.iter().next().cloned();

		match change {
			Change::Push(fd, id) => {
				lock.push_front((fd, id));
			}
			Change::Remove(fd, id) => {
				lock.retain(|&x| x != (fd, id));
			}
			Change::RemoveAll(fd) => {
				lock.retain(|&(x, _)| x != fd);
			}
		}

		let current = lock.iter().next().cloned();

		let mut clients = clients();

		eprintln!(
			"{:?}, {:?}",
			old.map(|x| (x.0, *x.1)),
			current.map(|x| (x.0, *x.1))
		);

		if old == current {
			return Ok(has_pending_messages);
		}

		if let Some((fd, id)) = old {
			let client = clients.get_mut(&fd).unwrap();

			let xdg_toplevel = client.get_object(id)?;
			let xdg_surface = client.get_object(xdg_toplevel.surface)?;
			let surface = client.get_object(xdg_surface.surface)?;

			for keyboard in client.objects_mut::<wl::Keyboard>() {
				keyboard.leave(client, surface.object_id)?;
			}

			xdg_toplevel.configure(client, 0, 0, &[])?;

			has_pending_messages = true;
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

			has_pending_messages = true;
		}
	}

	Ok(has_pending_messages)
}

pub fn add_change(change: Change) {
	changes().push(change);
}
