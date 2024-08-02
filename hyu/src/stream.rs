// TODO: get rid of
#[derive(Clone)]
pub struct Stream {
	stream: std::sync::Arc<std::cell::SyncUnsafeCell<std::os::unix::net::UnixStream>>,
}

impl Stream {
	pub fn new(stream: std::os::unix::net::UnixStream) -> Self {
		Self {
			stream: std::sync::Arc::new(std::cell::SyncUnsafeCell::new(stream)),
		}
	}

	pub fn get(&self) -> &mut std::os::unix::net::UnixStream {
		unsafe { &mut *self.stream.get() }
	}
}
