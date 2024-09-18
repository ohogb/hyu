use crate::{
	rt::{producers, Producer},
	Result,
};

#[derive(Clone)]
pub struct Sender<T> {
	sender: std::sync::mpsc::Sender<T>,
	event: producers::Notifier,
}

impl<T: 'static> Sender<T> {
	pub fn send(&self, value: T) -> Result<()> {
		self.sender
			.send(value)
			.map_err(|_| color_eyre::eyre::eyre!("failed to send value"))?;

		self.event.notify()
	}
}

pub struct Channel<T> {
	receiver: std::sync::mpsc::Receiver<T>,
	event: producers::EventFd,
}

impl<T> Channel<T> {
	pub fn new() -> Result<(Sender<T>, Self)> {
		let (tx, rx) = std::sync::mpsc::channel();
		let (a, b) = producers::EventFd::new()?;

		Ok((
			Sender {
				sender: tx,
				event: a,
			},
			Self {
				receiver: rx,
				event: b,
			},
		))
	}
}

impl<T> Producer for Channel<T> {
	type Message<'a> = T;
	type Ret = Result<()>;

	fn fd(&self) -> std::os::fd::RawFd {
		self.event.fd()
	}

	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>> {
		let a = self.event.read()?;

		for _ in 0..a {
			let x = self.receiver.try_recv()?;
			callback(x)?;
		}

		Ok(std::ops::ControlFlow::Continue(()))
	}
}
