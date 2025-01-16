#[link(name = "xkbcommon")]
unsafe extern "C" {
	fn xkb_context_new(flags: i32) -> Option<Context>;
	fn xkb_context_unref(context: usize);
}

#[repr(transparent)]
pub struct Context {
	ptr: std::num::NonZeroUsize,
}

impl Context {
	pub fn create() -> Option<Self> {
		unsafe { xkb_context_new(0) }
	}

	pub fn as_ptr(&self) -> usize {
		self.ptr.get()
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			xkb_context_unref(self.as_ptr());
		}
	}
}
