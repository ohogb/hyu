// TODO: remove this
pub struct GlobalWrapper<T> {
	value: std::cell::SyncUnsafeCell<Option<T>>,
}

impl<T> GlobalWrapper<T> {
	pub const fn empty() -> Self {
		Self {
			value: std::cell::SyncUnsafeCell::new(None),
		}
	}

	pub unsafe fn initialize(&self, value: T) {
		unsafe {
			*self.value.get() = Some(value);
		}
	}

	pub fn as_mut_ptr(&self) -> *mut T {
		unsafe { (*self.value.get()).as_mut().unwrap() as _ }
	}
}

impl<T> std::ops::Deref for GlobalWrapper<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { (*self.value.get()).as_ref().unwrap() }
	}
}

unsafe impl<T> Send for GlobalWrapper<T> {}
unsafe impl<T> Sync for GlobalWrapper<T> {}
