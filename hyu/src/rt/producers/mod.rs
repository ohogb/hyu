mod channel;
mod drm;
mod event_fd;
mod wl;

pub use channel::*;
pub use drm::*;
pub use event_fd::*;
pub use wl::*;

use crate::Result;

pub trait Producer {
	type Message<'a>;
	type Ret;

	fn fd(&self) -> std::os::fd::RawFd;
	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>>;
}
