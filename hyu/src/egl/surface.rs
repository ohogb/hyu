#[derive(Debug)]
#[repr(transparent)]
pub struct Surface {
	ptr: std::num::NonZeroU64,
}

impl Surface {
	pub fn as_ptr(&self) -> u64 {
		self.ptr.get()
	}
}
