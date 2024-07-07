#[derive(Debug)]
#[repr(transparent)]
pub struct Surface {
	ptr: std::ptr::NonNull<()>,
}

impl Surface {
	pub fn as_ptr(&self) -> u64 {
		self.ptr.as_ptr() as _
	}
}
