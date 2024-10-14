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
	epoll: nix::sys::epoll::Epoll,
	_phantom: std::marker::PhantomData<State>,
}

impl<State: 'static> EventLoop<State> {
	pub fn create() -> Result<Self> {
		Ok(Self {
			map: Default::default(),
			epoll: nix::sys::epoll::Epoll::new(nix::sys::epoll::EpollCreateFlags::empty())?,
			_phantom: std::marker::PhantomData,
		})
	}

	pub fn on<T: elp::Source + 'static>(
		&mut self,
		producer: T,
		callback: impl FnMut(T::Message<'_>, &mut State, &mut Self) -> T::Ret + 'static,
	) -> Result<()> {
		let fd = producer.fd();

		let a = Caller {
			producer,
			callback,
			_phantom: std::marker::PhantomData::<State>,
		};

		self.epoll.add(
			unsafe { std::os::fd::BorrowedFd::borrow_raw(fd) },
			nix::sys::epoll::EpollEvent::new(nix::sys::epoll::EpollFlags::EPOLLIN, fd as _),
		)?;

		self.map
			.insert(fd, std::rc::Rc::new(std::cell::RefCell::new(a)));

		Ok(())
	}

	pub fn run(&mut self, state: &mut State) -> Result<()> {
		loop {
			let mut events = [nix::sys::epoll::EpollEvent::empty()];
			let ret = self
				.epoll
				.wait(&mut events, nix::sys::epoll::EpollTimeout::NONE)?;

			assert!(ret == 1);
			let fd = events[0].data() as std::os::fd::RawFd;

			let entry = std::rc::Rc::clone(self.map.get_mut(&fd).unwrap());
			let mut entry = entry.borrow_mut();

			let ret = entry.call(state, self)?;

			match ret {
				std::ops::ControlFlow::Continue(_) => {}
				std::ops::ControlFlow::Break(_) => {
					self.map.remove(&fd).unwrap();
					self.epoll
						.delete(unsafe { std::os::fd::BorrowedFd::borrow_raw(fd) })?;
				}
			}
		}
	}
}
