use crate::{wl, Result, State};

pub struct Client {
	objects: std::collections::HashMap<u32, std::cell::UnsafeCell<wl::Resource>>,
	state: State,
	fds: std::collections::VecDeque<std::os::fd::RawFd>,
	pub windows: Vec<u32>,
}

impl Client {
	pub fn new(state: State) -> Self {
		Self {
			objects: std::collections::HashMap::new(),
			state,
			fds: Default::default(),
			windows: Vec::new(),
		}
	}

	pub fn push_client_object(&mut self, id: u32, object: impl Into<wl::Resource>) {
		self.objects
			.insert(id, std::cell::UnsafeCell::new(object.into()));
	}

	pub fn remove_client_object(&mut self, id: u32) -> Result<()> {
		let _ret = self.objects.remove(&id);
		// assert!(ret.is_some());

		self.send_message(wlm::Message {
			object_id: 1,
			op: 1,
			args: id,
		})?;

		Ok(())
	}

	pub fn get_object(&self, id: u32) -> Option<&'static wl::Resource> {
		self.objects.get(&id).map(|x| unsafe { &*x.get() })
	}

	pub fn get_object_mut(&self, id: u32) -> Option<&'static mut wl::Resource> {
		self.objects.get(&id).map(|x| unsafe { &mut *x.get() })
	}

	pub fn get_state(&mut self) -> &mut State {
		&mut self.state
	}

	pub fn send_message<T: serde::Serialize>(&mut self, message: wlm::Message<T>) -> Result<()> {
		self.get_state().buffer.0.extend(message.to_vec()?);
		Ok(())
	}

	pub fn push_fds(&mut self, fds: Vec<std::os::fd::RawFd>) {
		self.fds.extend(fds);
	}

	pub fn pop_fd(&mut self) -> std::os::fd::RawFd {
		self.fds.pop_front().unwrap()
	}

	pub fn add_window(&mut self, toplevel: u32) {
		self.windows.push(toplevel);
	}
}
