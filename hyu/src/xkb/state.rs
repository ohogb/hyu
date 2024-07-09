use crate::xkb;

#[link(name = "xkbcommon")]
extern "C" {
	fn xkb_state_new(keymap: u64) -> Option<State>;
	fn xkb_state_unref(state: u64);
	fn xkb_state_update_key(state: u64, key: u32, direction: i32) -> i32;
	fn xkb_state_serialize_mods(state: u64, components: i32) -> u32;
}

#[repr(transparent)]
pub struct State {
	ptr: std::num::NonZeroU64,
}

impl State {
	pub fn new(keymap: &xkb::Keymap) -> Option<Self> {
		unsafe { xkb_state_new(keymap.as_ptr()) }
	}

	pub fn update_key(&self, key: u32, state: i32) -> i32 {
		unsafe { xkb_state_update_key(self.as_ptr(), key, state) }
	}

	pub fn serialize_mods(&self, components: i32) -> u32 {
		unsafe { xkb_state_serialize_mods(self.as_ptr(), components) }
	}

	pub fn as_ptr(&self) -> u64 {
		self.ptr.get()
	}
}

impl Drop for State {
	fn drop(&mut self) {
		unsafe {
			xkb_state_unref(self.as_ptr());
		}
	}
}
