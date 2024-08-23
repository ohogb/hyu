use crate::{rt::Producer, Result};

pub struct Input {
	context: crate::backend::input::Context,
}

impl Input {
	pub fn new(context: crate::backend::input::Context) -> Self {
		Self { context }
	}
}

pub enum InputMessage {
	Event { event: crate::backend::input::Event },
}

impl Producer for Input {
	type Message<'a> = InputMessage;
	type Ret = Result<()>;

	fn fd(&self) -> std::os::fd::RawFd {
		self.context.get_fd()
	}

	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>> {
		self.context.dispatch();

		while let Some(event) = self.context.get_event() {
			callback(InputMessage::Event { event })?;
		}

		Ok(std::ops::ControlFlow::Continue(()))
	}
}
