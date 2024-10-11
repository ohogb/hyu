#[link(name = "udev")]
extern "C" {
	fn udev_new() -> Option<Instance>;
}

#[repr(transparent)]
pub struct Instance {
	ptr: std::ptr::NonNull<()>,
}

impl Instance {
	pub fn create() -> Option<Self> {
		unsafe { udev_new() }
	}
}
