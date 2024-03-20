use crate::{wl, Result, State};

enum ObjectChange {
	Add { id: u32, resource: wl::Resource },
	Remove { id: u32 },
}

pub struct Client<'object> {
	objects: Vec<Option<std::cell::UnsafeCell<wl::Resource>>>,
	object_queue: Vec<Option<ObjectChange>>,
	state: State,
	pub received_fds: std::collections::VecDeque<std::os::fd::RawFd>,
	pub to_send_fds: Vec<std::os::fd::RawFd>,
	pub windows: Vec<u32>,
	pub surface_cursor_is_over: Option<(u32, (i32, i32))>,
	pub has_keyboard_focus: bool,
	_phantom: std::marker::PhantomData<&'object ()>,
}

impl<'object> Client<'object> {
	pub fn new(state: State) -> Self {
		Self {
			objects: Vec::new(),
			object_queue: Vec::new(),
			state,
			received_fds: Default::default(),
			to_send_fds: Default::default(),
			windows: Vec::new(),
			surface_cursor_is_over: None,
			has_keyboard_focus: false,
			_phantom: std::marker::PhantomData,
		}
	}

	pub fn queue_new_object(&mut self, id: u32, object: impl Into<wl::Resource>) {
		self.object_queue.push(Some(ObjectChange::Add {
			id,
			resource: object.into(),
		}));
	}

	pub fn queue_remove_object(&mut self, id: u32) {
		self.object_queue.push(Some(ObjectChange::Remove { id }));
	}

	pub fn process_queue(&mut self) -> Result<()> {
		let mut changes = std::mem::take(&mut self.object_queue);

		for change in &mut changes {
			let change = std::mem::take(change);
			let change = change.expect("`None` shouldn't exist in `object_changes`");

			match change {
				ObjectChange::Add { id, resource } => {
					if self.objects.len() < (id + 1) as usize {
						self.objects.resize_with((id + 1) as _, Default::default);
					}

					self.objects[id as usize] = Some(std::cell::UnsafeCell::new(resource));
				}
				ObjectChange::Remove { id } => {
					// TODO: make sure `id` exists

					if let Some(a) = self.objects.get_mut(id as usize) {
						*a = None;
					}

					self.send_message(wlm::Message {
						object_id: 1,
						op: 1,
						args: id,
					})?;
				}
			}
		}

		self.object_queue = changes;
		self.object_queue.clear();

		Ok(())
	}

	pub fn get_object<T>(&self, id: u32) -> Result<&'object T>
	where
		Result<&'object T>: From<&'object wl::Resource>,
	{
		self.get_resource(id)
			.ok_or_else(|| format!("object '{id}' does not exist"))?
			.into()
	}

	pub fn get_object_mut<T>(&self, id: u32) -> Result<&'object mut T>
	where
		Result<&'object mut T>: From<&'object mut wl::Resource>,
	{
		self.get_resource_mut(id)
			.ok_or_else(|| format!("object '{id}' does not exist"))?
			.into()
	}

	pub fn get_resource(&self, id: u32) -> Option<&'object wl::Resource> {
		self.objects
			.get(id as usize)
			.and_then(|x| x.as_ref().map(|x| unsafe { &*x.get() }))
	}

	pub fn get_resource_mut(&self, id: u32) -> Option<&'object mut wl::Resource> {
		self.objects
			.get(id as usize)
			.and_then(|x| x.as_ref().map(|x| unsafe { &mut *x.get() }))
	}

	pub fn get_state(&mut self) -> &mut State {
		&mut self.state
	}

	pub fn send_message<T: serde::Serialize>(&mut self, message: wlm::Message<T>) -> Result<()> {
		self.get_state().buffer.0.extend(message.to_vec()?);
		Ok(())
	}

	pub fn add_window(&mut self, toplevel: u32) {
		self.windows.push(toplevel);
	}

	pub fn objects(&self) -> Vec<&'static mut wl::Resource> {
		self.objects
			.iter()
			.filter_map(|x| x.as_ref().map(|x| unsafe { &mut *x.get() }))
			.collect()
	}
}
