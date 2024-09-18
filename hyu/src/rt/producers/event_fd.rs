use std::os::fd::AsRawFd as _;

use crate::{rt::Producer, Result};

#[derive(Clone)]
pub struct Notifier(std::sync::Arc<nix::sys::eventfd::EventFd>);

impl Notifier {
	pub fn notify(&self) -> Result<()> {
		self.0.write(1)?;
		Ok(())
	}
}

pub struct EventFd(std::sync::Arc<nix::sys::eventfd::EventFd>);

impl EventFd {
	pub fn new() -> Result<(Notifier, Self)> {
		let a = std::sync::Arc::new(nix::sys::eventfd::EventFd::new()?);
		Ok((Notifier(a.clone()), Self(a)))
	}

	pub fn read(&mut self) -> Result<u64> {
		Ok(self.0.read()?)
	}
}

impl Producer for EventFd {
	type Message<'a> = ();
	type Ret = Result<()>;

	fn fd(&self) -> std::os::fd::RawFd {
		self.0.as_raw_fd()
	}

	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>> {
		self.read()?;
		callback(())?;

		Ok(std::ops::ControlFlow::Continue(()))
	}
}
