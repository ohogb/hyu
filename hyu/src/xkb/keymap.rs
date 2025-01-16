use crate::xkb;

#[link(name = "xkbcommon")]
unsafe extern "C" {
	fn xkb_keymap_new_from_names(context: usize, names: usize, flags: i32) -> Option<Keymap>;
	fn xkb_keymap_unref(keymap: usize);
	fn xkb_keymap_get_as_string(keymap: usize, format: i32) -> u64;
}

#[repr(transparent)]
pub struct Keymap {
	ptr: std::num::NonZeroUsize,
}

impl Keymap {
	pub fn create(context: &xkb::Context, layout: impl AsRef<str>) -> Option<Self> {
		let layout = std::ffi::CString::new(layout.as_ref()).unwrap();

		let mut rule_names = [0u64; 5];
		rule_names[2] = layout.as_ptr() as _;

		unsafe { xkb_keymap_new_from_names(context.as_ptr(), rule_names.as_ptr() as _, 0) }
	}

	pub fn get_as_string(&self) -> String {
		unsafe {
			let ret = xkb_keymap_get_as_string(self.as_ptr(), 1);
			let str = std::ffi::CStr::from_ptr(ret as _);

			// TODO: free the pointer?
			str.to_str().unwrap().to_string()
		}
	}

	pub fn as_ptr(&self) -> usize {
		self.ptr.get()
	}
}

impl Drop for Keymap {
	fn drop(&mut self) {
		unsafe {
			xkb_keymap_unref(self.as_ptr());
		}
	}
}
