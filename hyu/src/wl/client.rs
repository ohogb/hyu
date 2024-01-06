use crate::{wl, Result, State};

pub struct Client {
	objects: std::collections::HashMap<u32, Box<dyn wl::Object>>,
	state: State,
	fds: std::collections::VecDeque<std::os::fd::RawFd>,
	windows: Vec<u32>,
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

	pub fn push_client_object(&mut self, id: u32, object: impl wl::Object + 'static) {
		self.objects.insert(id, Box::new(object));
	}

	pub fn get_object_mut(&mut self, id: u32) -> Option<&mut Box<dyn wl::Object>> {
		self.objects.get_mut(&id)
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

	pub fn get_windows(&mut self) -> Vec<*mut wl::XdgToplevel> {
		self.windows
			.clone()
			.iter()
			.map(|x| self.get_object_mut(*x).unwrap().as_mut() as *mut _ as *mut wl::XdgToplevel)
			.collect()
	}
}
