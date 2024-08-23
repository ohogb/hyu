use crate::{wl, Point, Result};

pub struct Client {
	pub fd: std::os::fd::RawFd,
	objects: Vec<Option<std::cell::UnsafeCell<wl::Resource>>>,
	pub start_position: Point,
	pub received_fds: std::collections::VecDeque<std::os::fd::RawFd>,
	pub to_send_fds: Vec<std::os::fd::RawFd>,
	highest_index: u32,
	pub stream: crate::Stream,
}

impl<'object> Client {
	pub fn new(fd: std::os::fd::RawFd, start_position: Point, stream: crate::Stream) -> Self {
		Self {
			fd,
			objects: Vec::new(),
			start_position,
			received_fds: Default::default(),
			to_send_fds: Default::default(),
			highest_index: 0,
			stream,
		}
	}

	pub fn ensure_objects_capacity(&mut self) {
		// TODO: cleanup this mess
		const THRESHOLD: isize = 10;

		if ((self.objects.len() as isize - self.highest_index as isize) - THRESHOLD) < 0 {
			self.objects.resize_with(
				(self.objects.len() + THRESHOLD as usize) * 2,
				Default::default,
			);
		}
	}

	pub fn new_object<T: Into<wl::Resource>>(&mut self, id: wl::Id<T>, object: T) -> &'object mut T
	where
		Result<&'object mut T>: From<&'object mut wl::Resource>,
	{
		assert!((*id as usize) < self.objects.len());
		assert!(self.objects[*id as usize].is_none());

		self.objects[*id as usize] = Some(std::cell::UnsafeCell::new(object.into()));

		if self.highest_index < *id {
			self.highest_index = *id;
		}

		self.get_object_mut(id).unwrap()
	}

	pub fn remove_object<T>(&mut self, id: wl::Id<T>) -> Result<()> {
		assert!(self.objects[*id as usize].is_some());
		// TODO: check that it's type T

		self.objects[*id as usize] = None;
		const DISPLAY_ID: wl::Id<wl::Display> = wl::Id::new(1);

		let display = self.get_object(DISPLAY_ID)?;
		display.delete_id(self, id)
	}

	pub fn get_object<T>(&self, id: wl::Id<T>) -> Result<&'object T>
	where
		Result<&'object T>: From<&'object wl::Resource>,
	{
		self.get_resource(*id)
			.ok_or_else(|| {
				format!(
					"object '{}@{}' does not exist",
					std::any::type_name::<T>(),
					*id
				)
			})?
			.into()
	}

	pub fn get_object_mut<T>(&self, id: wl::Id<T>) -> Result<&'object mut T>
	where
		Result<&'object mut T>: From<&'object mut wl::Resource>,
	{
		self.get_resource_mut(*id)
			.ok_or_else(|| {
				format!(
					"object '{}@{}' does not exist",
					std::any::type_name::<T>(),
					*id
				)
			})?
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
		let mut cmsg_buffer = [0u8; 0x20];
		let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

		cmsg.add_fds(&self.to_send_fds);
		self.to_send_fds.clear();

		let ret = self
			.stream
			.get()
			.send_vectored_with_ancillary(&[std::io::IoSlice::new(&message.to_vec()?)], &mut cmsg);

		if ret.is_err() {
			eprintln!("Client::send_message() failed!");
		}

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
