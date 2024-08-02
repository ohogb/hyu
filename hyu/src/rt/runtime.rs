use std::os::fd::{AsFd as _, AsRawFd as _};

use crate::{rt::Producer, Result};

struct Caller<T: Producer, U, V: FnMut(T::Message<'_>, &mut U) -> T::Ret> {
	producer: T,
	callback: V,
	_phantom: std::marker::PhantomData<U>,
}

trait CallerWrapper<T> {
	fn call(&mut self, state: &mut T) -> Result<std::ops::ControlFlow<()>>;
}

impl<T: Producer, U, V: FnMut(T::Message<'_>, &mut U) -> T::Ret> CallerWrapper<U>
	for Caller<T, U, V>
{
	fn call(&mut self, state: &mut U) -> Result<std::ops::ControlFlow<()>> {
		self.producer
			.call(&mut |event| (self.callback)(event, state))
	}
}

pub struct Runtime<State> {
	map: std::collections::HashMap<std::os::fd::RawFd, Box<dyn CallerWrapper<State>>>,
	_phantom: std::marker::PhantomData<State>,
}

impl<State: 'static> Runtime<State> {
	pub fn new() -> Self {
		Self {
			map: Default::default(),
			_phantom: std::marker::PhantomData,
		}
	}

	pub fn on<T: Producer + 'static>(
		&mut self,
		producer: T,
		callback: impl FnMut(T::Message<'_>, &mut State) -> T::Ret + 'static,
	) {
		let fd = producer.fd();

		let a = Caller {
			producer,
			callback,
			_phantom: std::marker::PhantomData::<State>,
		};

		self.map.insert(fd, Box::new(a));
	}

	pub fn run(&mut self, state: &mut State) -> Result<()> {
		'outer: loop {
			// TODO: clean this up
			let mut keys = self
				.map
				.keys()
				.map(|&x| {
					nix::poll::PollFd::new(
						unsafe { std::os::fd::BorrowedFd::borrow_raw(x) },
						nix::poll::PollFlags::POLLIN,
					)
				})
				.collect::<Vec<_>>();

			let _ = nix::poll::poll(&mut keys, nix::poll::PollTimeout::NONE)?;
			for i in keys {
				if !i.any().unwrap() {
					continue;
				}

				let entry = self.map.get_mut(&i.as_fd().as_raw_fd()).unwrap();
				let ret = entry.call(state)?;

				match ret {
					std::ops::ControlFlow::Continue(_) => {}
					std::ops::ControlFlow::Break(_) => break 'outer,
				}
			}
		}

		Ok(())
	}
}
