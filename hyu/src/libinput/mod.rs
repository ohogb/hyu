use crate::udev;

#[link(name = "input")]
unsafe extern "C" {
	fn libinput_udev_create_context(
		interface: usize,
		user_data: usize,
		udev: usize,
	) -> Option<Context>;

	fn libinput_udev_assign_seat(context: usize, name: usize) -> i32;
	fn libinput_get_fd(context: usize) -> std::os::fd::RawFd;
	fn libinput_dispatch(context: usize) -> i32;
	fn libinput_get_event(context: usize) -> Option<Event>;
	fn libinput_event_get_type(event: usize) -> i32;
	fn libinput_event_get_keyboard_event(event: usize) -> Option<EventKeyboard>;
	fn libinput_event_get_pointer_event(event: usize) -> Option<EventPointer>;
	fn libinput_event_keyboard_get_key(event_keyboard: usize) -> u32;
	fn libinput_event_keyboard_get_key_state(event_keyboard: usize) -> i32;
	fn libinput_event_pointer_get_dx_unaccelerated(event_pointer: usize) -> f64;
	fn libinput_event_pointer_get_dy_unaccelerated(event_pointer: usize) -> f64;
	fn libinput_event_pointer_get_button(event_pointer: usize) -> u32;
	fn libinput_event_pointer_get_button_state(event_pointer: usize) -> u32;
	fn libinput_event_pointer_get_scroll_value_v120(event_pointer: usize, axis: u32) -> f64;
}

#[repr(transparent)]
pub struct Context {
	ptr: std::num::NonZeroUsize,
}

impl Context {
	pub fn create_from_udev(udev: &udev::Instance) -> Option<Self> {
		extern "C" fn open(path: *const i8, flags: i32, _user_data: usize) -> i32 {
			unsafe { nix::libc::open(path, flags) }
		}

		extern "C" fn close(fd: std::os::fd::RawFd, _user_data: usize) {
			unsafe { nix::libc::close(fd) };
		}

		let interface = Box::leak(Box::new([open as usize, close as usize]));

		unsafe { libinput_udev_create_context(interface.as_ptr() as _, 0, udev.as_ptr()) }
	}

	pub fn assign(&self) -> i32 {
		unsafe { libinput_udev_assign_seat(self.as_ptr(), c"seat0".as_ptr() as _) }
	}

	pub fn get_fd(&self) -> std::os::fd::RawFd {
		unsafe { libinput_get_fd(self.as_ptr()) }
	}

	pub fn dispatch(&self) -> i32 {
		unsafe { libinput_dispatch(self.as_ptr()) }
	}

	pub fn get_event(&self) -> Option<Event> {
		unsafe { libinput_get_event(self.as_ptr()) }
	}

	pub fn as_ptr(&self) -> usize {
		self.ptr.get()
	}
}

#[repr(transparent)]
pub struct Event {
	ptr: std::num::NonZeroUsize,
}

impl Event {
	pub fn get_type(&self) -> i32 {
		unsafe { libinput_event_get_type(self.as_ptr()) }
	}

	pub fn get_keyboard_event(&self) -> Option<EventKeyboard> {
		unsafe { libinput_event_get_keyboard_event(self.as_ptr()) }
	}

	pub fn get_pointer_event(&self) -> Option<EventPointer> {
		unsafe { libinput_event_get_pointer_event(self.as_ptr()) }
	}

	pub fn as_ptr(&self) -> usize {
		self.ptr.get()
	}
}

#[repr(transparent)]
pub struct EventKeyboard {
	ptr: std::num::NonZeroUsize,
}

impl EventKeyboard {
	pub fn get_key(&self) -> u32 {
		unsafe { libinput_event_keyboard_get_key(self.as_ptr()) }
	}

	pub fn get_key_state(&self) -> i32 {
		unsafe { libinput_event_keyboard_get_key_state(self.as_ptr()) }
	}

	pub fn as_ptr(&self) -> usize {
		self.ptr.get()
	}
}

#[repr(transparent)]
pub struct EventPointer {
	ptr: std::num::NonZeroUsize,
}

impl EventPointer {
	pub fn get_dx(&self) -> f64 {
		unsafe { libinput_event_pointer_get_dx_unaccelerated(self.as_ptr()) }
	}

	pub fn get_dy(&self) -> f64 {
		unsafe { libinput_event_pointer_get_dy_unaccelerated(self.as_ptr()) }
	}

	pub fn get_button(&self) -> u32 {
		unsafe { libinput_event_pointer_get_button(self.as_ptr()) }
	}

	pub fn get_button_state(&self) -> u32 {
		unsafe { libinput_event_pointer_get_button_state(self.as_ptr()) }
	}

	pub fn get_scroll_value_v120(&self, axis: u32) -> f64 {
		unsafe { libinput_event_pointer_get_scroll_value_v120(self.as_ptr(), axis) }
	}

	pub fn as_ptr(&self) -> usize {
		self.ptr.get()
	}
}
