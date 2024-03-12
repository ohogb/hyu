use crate::{wl, Result, State};

pub struct Client {
	objects: Vec<Option<std::cell::UnsafeCell<wl::Resource>>>,
	state: State,
	fds: std::collections::VecDeque<std::os::fd::RawFd>,
	pub windows: Vec<u32>,
	pub surface_cursor_is_over: Option<(u32, (i32, i32))>,
	pub has_keyboard_focus: bool,
}

impl Client {
	pub fn new(state: State) -> Self {
		Self {
			objects: Vec::new(),
			state,
			fds: Default::default(),
			windows: Vec::new(),
			surface_cursor_is_over: None,
			has_keyboard_focus: false,
		}
	}

	pub fn push_client_object(&mut self, id: u32, object: impl Into<wl::Resource>) {
		if self.objects.len() < (id + 1) as usize {
			self.objects.resize_with((id + 1) as _, Default::default);
		}

		self.objects[id as usize] = Some(std::cell::UnsafeCell::new(object.into()));
	}

	pub fn remove_client_object(&mut self, id: u32) -> Result<()> {
		if let Some(a) = self.objects.get_mut(id as usize) {
			*a = None;
		}
		// let _ret = self.objects.remove(&id);
		// assert!(ret.is_some());

		self.send_message(wlm::Message {
			object_id: 1,
			op: 1,
			args: id,
		})?;

		Ok(())
	}

	pub fn get_object<T>(&self, id: u32) -> Result<&'static T>
	where
		Result<&'static T>: From<&'static wl::Resource>,
	{
		self.get_resource(id)
			.ok_or_else(|| format!("object '{id}' does not exist"))?
			.into()
	}

	pub fn get_object_mut<T>(&self, id: u32) -> Result<&'static mut T>
	where
		Result<&'static mut T>: From<&'static mut wl::Resource>,
	{
		self.get_resource_mut(id)
			.ok_or_else(|| format!("object '{id}' does not exist"))?
			.into()
	}

	pub fn get_resource(&self, id: u32) -> Option<&'static wl::Resource> {
		self.objects
			.get(id as usize)
			.map(|x| x.as_ref().map(|x| unsafe { &*x.get() }))
			.flatten()
	}

	pub fn get_resource_mut(&self, id: u32) -> Option<&'static mut wl::Resource> {
		self.objects
			.get(id as usize)
			.map(|x| x.as_ref().map(|x| unsafe { &mut *x.get() }))
			.flatten()
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

	pub fn objects(&self) -> Vec<&'static mut wl::Resource> {
		self.objects
			.iter()
			.map(|x| x.as_ref().map(|x| unsafe { &mut *x.get() }))
			.flatten()
			.collect()
	}
}
