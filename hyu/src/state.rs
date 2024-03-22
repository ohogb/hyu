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
