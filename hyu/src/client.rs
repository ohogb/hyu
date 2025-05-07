use crate::{Point, Result, state, wl};

pub struct Client {
	pub fd: std::os::fd::RawFd,
	client_store: crate::Store<1>,
	pub start_position: Point,
	pub received_fds: std::collections::VecDeque<std::os::fd::RawFd>,
	pub to_send_fds: Vec<std::os::fd::RawFd>,
	pub changes: Vec<state::Change>,
}

impl<'object> Client {
	pub fn new(fd: std::os::fd::RawFd, start_position: Point) -> Self {
		Self {
			fd,
			client_store: Default::default(),
			start_position,
			received_fds: Default::default(),
			to_send_fds: Default::default(),
			changes: Vec::new(),
		}
	}

	pub fn ensure_objects_capacity(&mut self) {
		self.client_store.ensure_objects_capacity();
	}

	pub fn new_object<T: Into<wl::Resource>>(&mut self, id: wl::Id<T>, object: T) -> &'object mut T
	where
		Result<&'object mut T>: From<&'object mut wl::Resource>,
	{
		self.client_store.new_object(id, object)
	}

	pub unsafe fn remove_object<T>(&mut self, id: wl::Id<T>) -> Result<()> {
		unsafe {
			self.client_store.remove_object(id)?;
		}

		const DISPLAY_ID: wl::Id<wl::Display> = wl::Id::new(1);

		let display = self.get_object(DISPLAY_ID)?;
		display.delete_id(id)
	}

	pub fn get_object<T>(&self, id: wl::Id<T>) -> Result<&'object T>
	where
		Result<&'object T>: From<&'object wl::Resource>,
	{
		self.client_store.get_object(id)
	}

	pub fn get_object_mut<T>(&self, id: wl::Id<T>) -> Result<&'object mut T>
	where
		Result<&'object mut T>: From<&'object mut wl::Resource>,
	{
		self.client_store.get_object_mut(id)
	}

	pub fn get_resource(&self, id: u32) -> Option<&'object wl::Resource> {
		self.client_store.get_resource(id)
	}

	pub fn get_resource_mut(&self, id: u32) -> Option<&'object mut wl::Resource> {
		self.client_store.get_resource_mut(id)
	}

	pub fn objects_mut<T>(&self) -> Vec<&'object mut T>
	where
		Result<&'object mut T>: From<&'object mut wl::Resource>,
	{
		self.client_store.objects_mut()
	}
}
