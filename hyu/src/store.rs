use crate::{Result, wl};

#[derive(Default)]
pub struct Store<const START: u32> {
	objects: Vec<Option<std::cell::UnsafeCell<wl::Resource>>>,
	highest_index: u32,
}

impl<'object, const START: u32> Store<START> {
	pub fn new() -> Self {
		Self {
			..Default::default()
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
		let index = Self::wl_id_to_index(id);

		assert!(index < self.objects.len());
		assert!(self.objects[index].is_none());

		self.objects[index] = Some(std::cell::UnsafeCell::new(object.into()));

		let index = index as u32;

		if self.highest_index < index {
			self.highest_index = index;
		}

		self.get_object_mut(id).unwrap()
	}

	pub unsafe fn remove_object<T>(&mut self, id: wl::Id<T>) -> Result<()> {
		let index = Self::wl_id_to_index(id);
		assert!(self.objects[index].is_some());
		// TODO: check that it's type T

		self.objects[index] = None;
		Ok(())
	}

	pub fn get_object<T>(&self, id: wl::Id<T>) -> Result<&'object T>
	where
		Result<&'object T>: From<&'object wl::Resource>,
	{
		self.get_resource(*id)
			.ok_or_else(|| {
				color_eyre::eyre::eyre!(
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
				color_eyre::eyre::eyre!(
					"object '{}@{}' does not exist",
					std::any::type_name::<T>(),
					*id
				)
			})?
			.into()
	}

	pub fn get_resource(&self, id: u32) -> Option<&'object wl::Resource> {
		self.objects
			.get(Self::id_to_index(id))
			.and_then(|x| x.as_ref().map(|x| unsafe { &*x.get() }))
	}

	pub fn get_resource_mut(&self, id: u32) -> Option<&'object mut wl::Resource> {
		self.objects
			.get(Self::id_to_index(id))
			.and_then(|x| x.as_ref().map(|x| unsafe { &mut *x.get() }))
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

	fn wl_id_to_index<T>(id: wl::Id<T>) -> usize {
		(*id - START) as usize
	}

	fn id_to_index(id: u32) -> usize {
		(id - START) as usize
	}
}
