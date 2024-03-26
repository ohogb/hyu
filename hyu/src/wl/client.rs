use crate::{wl, Result};

enum ObjectChange {
	Add { id: u32, resource: wl::Resource },
	Remove { id: u32 },
}

pub struct Client<'object> {
	pub fd: std::os::fd::RawFd,
	objects: Vec<Option<std::cell::UnsafeCell<wl::Resource>>>,
	object_queue: Vec<Option<ObjectChange>>,
	pub buffer: Vec<u8>,
	pub start_position: (i32, i32),
	pub received_fds: std::collections::VecDeque<std::os::fd::RawFd>,
	pub to_send_fds: Vec<std::os::fd::RawFd>,
	_phantom: std::marker::PhantomData<&'object ()>,
}

impl<'object> Client<'object> {
	pub fn new(fd: std::os::fd::RawFd, start_position: (i32, i32)) -> Self {
		Self {
			fd,
			objects: Vec::new(),
			object_queue: Vec::new(),
			buffer: Vec::new(),
			start_position,
			received_fds: Default::default(),
			to_send_fds: Default::default(),
			_phantom: std::marker::PhantomData,
		}
	}

	pub fn queue_new_object<T: Into<wl::Resource>>(&mut self, id: wl::Id<T>, object: T) {
		self.object_queue.push(Some(ObjectChange::Add {
			id: *id,
			resource: object.into(),
		}));
	}

	pub fn queue_remove_object<T>(&mut self, id: wl::Id<T>) {
		self.object_queue
			.push(Some(ObjectChange::Remove { id: *id }));
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

	pub fn get_object<T>(&self, id: wl::Id<T>) -> Result<&'object T>
	where
		Result<&'object T>: From<&'object wl::Resource>,
	{
		self.get_resource(*id)
			.ok_or_else(|| format!("object '{}' does not exist", *id))?
			.into()
	}

	pub fn get_object_mut<T>(&self, id: wl::Id<T>) -> Result<&'object mut T>
	where
		Result<&'object mut T>: From<&'object mut wl::Resource>,
	{
		self.get_resource_mut(*id)
			.ok_or_else(|| format!("object '{}' does not exist", *id))?
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

	pub fn send_message<T: serde::Serialize>(&mut self, message: wlm::Message<T>) -> Result<()> {
		self.buffer.extend(message.to_vec()?);
		Ok(())
	}

	pub fn objects_mut<T>(&self) -> Vec<&'object mut T>
	where
		Result<&'object mut T>: From<&'object mut wl::Resource>,
	{
		self.objects
			.iter()
			.filter_map(|x| x.as_ref().map(|x| unsafe { &mut *x.get() }))
			.map(Result::from)
			.filter_map(|x| x.ok())
			.collect()
	}
}
