use crate::wl;

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
	std::sync::Mutex<std::collections::VecDeque<(std::os::fd::RawFd, u32)>>,
> = std::sync::OnceLock::new();

pub fn window_stack(
) -> std::sync::MutexGuard<'static, std::collections::VecDeque<(std::os::fd::RawFd, u32)>> {
	WINDOW_STACK.get_or_init(Default::default).lock().unwrap()
}
