use std::os::fd::{AsFd as _, AsRawFd as _};

use crate::{elp, Result};

struct Caller<T: elp::Source, U, V: FnMut(T::Message<'_>, &mut U, &mut EventLoop<U>) -> T::Ret> {
	producer: T,
	callback: V,
	_phantom: std::marker::PhantomData<U>,
}

trait CallerWrapper<T> {
	fn call(&mut self, state: &mut T, rt: &mut EventLoop<T>) -> Result<std::ops::ControlFlow<()>>;
}

impl<T: elp::Source, U, V: FnMut(T::Message<'_>, &mut U, &mut EventLoop<U>) -> T::Ret>
	CallerWrapper<U> for Caller<T, U, V>
{
	fn call(&mut self, state: &mut U, rt: &mut EventLoop<U>) -> Result<std::ops::ControlFlow<()>> {
		self.producer
			.call(&mut |event| (self.callback)(event, state, rt))
	}
}

pub struct EventLoop<State> {
	map: std::collections::HashMap<
		std::os::fd::RawFd,
		std::rc::Rc<std::cell::RefCell<dyn CallerWrapper<State>>>,
	>,
	_phantom: std::marker::PhantomData<State>,
}

impl<State: 'static> EventLoop<State> {
	pub fn new() -> Self {
		Self {
			map: Default::default(),
			_phantom: std::marker::PhantomData,
		}
	}

	pub fn on<T: elp::Source + 'static>(
		&mut self,
		producer: T,
		callback: impl FnMut(T::Message<'_>, &mut State, &mut Self) -> T::Ret + 'static,
	) {
		let fd = producer.fd();

		let a = Caller {
			producer,
			callback,
			_phantom: std::marker::PhantomData::<State>,
		};

		self.map
			.insert(fd, std::rc::Rc::new(std::cell::RefCell::new(a)));
	}

	pub fn run(&mut self, state: &mut State) -> Result<()> {
		loop {
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

				let entry = std::rc::Rc::clone(self.map.get_mut(&i.as_fd().as_raw_fd()).unwrap());
				let mut entry = entry.borrow_mut();

				let ret = entry.call(state, self)?;

				match ret {
					std::ops::ControlFlow::Continue(_) => {}
					std::ops::ControlFlow::Break(_) => {
						self.map.remove(&i.as_fd().as_raw_fd()).unwrap();
					}
				}
			}
		}
	}
}
