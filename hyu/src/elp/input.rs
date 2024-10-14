use crate::{elp, libinput, Result};

pub struct Source {
	context: libinput::Context,
}

pub enum Message {
	Event { event: libinput::Event },
}

impl elp::Source for Source {
	type Message<'a> = Message;
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
			callback(Message::Event { event })?;
		}

		Ok(std::ops::ControlFlow::Continue(()))
	}
}

pub fn create(context: libinput::Context) -> Source {
	Source { context }
}
