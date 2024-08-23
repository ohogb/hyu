use std::os::fd::AsRawFd as _;

use crate::{rt::Producer, Result};

pub struct UnixListener {
	listener: std::os::unix::net::UnixListener,
}

impl UnixListener {
	pub fn new(listener: std::os::unix::net::UnixListener) -> Self {
		Self { listener }
	}
}

impl Producer for UnixListener {
	type Message<'a> = (
		std::os::unix::net::UnixStream,
		std::os::unix::net::SocketAddr,
	);

	type Ret = Result<()>;

	fn fd(&self) -> std::os::fd::RawFd {
		self.listener.as_raw_fd()
	}

	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>> {
		let ret = self.listener.accept()?;
		callback(ret)?;

		Ok(std::ops::ControlFlow::Continue(()))
	}
}
