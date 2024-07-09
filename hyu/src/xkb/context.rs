#[link(name = "xkbcommon")]
extern "C" {
	fn xkb_context_new(flags: i32) -> Option<Context>;
	fn xkb_context_unref(context: u64);
}

#[repr(transparent)]
pub struct Context {
	ptr: std::num::NonZeroU64,
}

impl Context {
	pub fn create() -> Option<Self> {
		unsafe { xkb_context_new(0) }
	}

	pub fn as_ptr(&self) -> u64 {
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
