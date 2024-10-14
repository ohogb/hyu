use std::os::fd::{AsFd as _, AsRawFd as _};

use crate::{elp, Result};

pub struct Source(std::sync::Arc<nix::sys::timerfd::TimerFd>);

impl Source {
	pub fn unset(&mut self) -> Result<()> {
		Ok(self.0.unset()?)
	}
}

impl elp::Source for Source {
	type Message<'a> = ();
	type Ret = Result<()>;

	fn fd(&self) -> std::os::fd::RawFd {
		self.0.as_fd().as_raw_fd()
	}

	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>> {
		self.unset()?;
		callback(())?;

		Ok(std::ops::ControlFlow::Continue(()))
	}
}

pub fn create() -> Result<(std::sync::Arc<nix::sys::timerfd::TimerFd>, Source)> {
	let a = std::sync::Arc::new(nix::sys::timerfd::TimerFd::new(
		nix::sys::timerfd::ClockId::CLOCK_MONOTONIC,
		nix::sys::timerfd::TimerFlags::TFD_NONBLOCK,
	)?);

	Ok((a.clone(), Source(a)))
}
