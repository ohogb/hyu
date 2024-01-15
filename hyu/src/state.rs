use crate::{wl, Vertex};

pub struct State {
	pub buffer: Buffer,
	pub start_position: (i32, i32),
}

pub struct Buffer(pub Vec<u8>);

static CLIENTS: std::sync::OnceLock<
	std::sync::Mutex<std::collections::HashMap<std::os::fd::RawFd, wl::Client>>,
> = std::sync::OnceLock::new();

pub fn clients(
) -> std::sync::MutexGuard<'static, std::collections::HashMap<std::os::fd::RawFd, wl::Client>> {
	CLIENTS
		.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
		.lock()
		.unwrap()
}

static VERTEX_BUFFER: std::sync::OnceLock<std::sync::Mutex<Vec<Vertex>>> =
	std::sync::OnceLock::new();

pub fn vertex_buffer() -> std::sync::MutexGuard<'static, Vec<Vertex>> {
	VERTEX_BUFFER
		.get_or_init(|| std::sync::Mutex::new(Vec::new()))
		.lock()
		.unwrap()
}
