use crate::Result;

pub trait Source {
	type Message<'a>;
	type Ret;

	fn fd(&self) -> std::os::fd::RawFd;
	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>>;
}
