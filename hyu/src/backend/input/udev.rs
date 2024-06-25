#[link(name = "udev")]
extern "C" {
	fn udev_new() -> Option<Instance>;
}

#[repr(transparent)]
pub struct Instance {
	ptr: std::ptr::NonNull<()>,
}

impl Instance {
	pub fn new() -> Self {
		unsafe { udev_new() }.unwrap()
	}
}
