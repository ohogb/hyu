use std::os::fd::AsRawFd as _;

use crate::{rt::Producer, Result};

pub struct EventFd(std::sync::Arc<nix::sys::eventfd::EventFd>);

impl EventFd {
	pub fn new() -> Result<(std::sync::Arc<nix::sys::eventfd::EventFd>, Self)> {
		let a = std::sync::Arc::new(nix::sys::eventfd::EventFd::new()?);
		Ok((a.clone(), Self(a)))
	}

	pub fn read(&mut self) -> Result<u64> {
		Ok(self.0.read()?)
	}
}

impl Producer for EventFd {
	type Message<'a> = ();
	type Ret = ();

	fn fd(&self) -> std::os::fd::RawFd {
		self.0.as_raw_fd()
	}

	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>> {
		self.read()?;
		callback(());

		Ok(std::ops::ControlFlow::Continue(()))
	}
}
