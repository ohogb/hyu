#[derive(Debug)]
#[repr(transparent)]
pub struct Config {
	ptr: std::ptr::NonNull<()>,
}

impl Config {
	pub fn as_ptr(&self) -> u64 {
		self.ptr.as_ptr() as _
	}
}
