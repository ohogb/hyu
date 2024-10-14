use std::os::fd::AsRawFd as _;

use crate::{elp, Result};

#[derive(Clone)]
pub struct Notifier(std::sync::Arc<nix::sys::eventfd::EventFd>);

impl Notifier {
	pub fn notify(&self) -> Result<()> {
		self.0.write(1)?;
		Ok(())
	}
}

pub struct Source(std::sync::Arc<nix::sys::eventfd::EventFd>);

impl Source {
	pub fn read(&mut self) -> Result<u64> {
		Ok(self.0.read()?)
	}
}

impl elp::Source for Source {
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

pub fn create() -> Result<(Notifier, Source)> {
	let a = std::sync::Arc::new(nix::sys::eventfd::EventFd::new()?);
	Ok((Notifier(a.clone()), Source(a)))
}
