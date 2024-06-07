#[derive(Debug)]
#[repr(transparent)]
pub struct Context {
	ptr: std::ptr::NonNull<()>,
}

impl Context {
	pub fn as_ptr(&self) -> u64 {
		self.ptr.as_ptr() as _
	}
}
