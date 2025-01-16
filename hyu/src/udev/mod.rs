#[link(name = "udev")]
unsafe extern "C" {
	fn udev_new() -> Option<Instance>;
	fn udev_unref(instance: usize);
}

#[repr(transparent)]
pub struct Instance {
	ptr: std::num::NonZeroUsize,
}

impl Instance {
	pub fn create() -> Option<Self> {
		unsafe { udev_new() }
	}
}

impl Drop for Instance {
	fn drop(&mut self) {
		unsafe {
			udev_unref(self.ptr.get());
		}
	}
}
