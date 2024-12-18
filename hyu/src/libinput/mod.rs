use crate::udev;

#[link(name = "input")]
unsafe extern "C" {
	fn libinput_udev_create_context(
		interface: u64,
		user_data: u64,
		udev: udev::Instance,
	) -> Option<Context>;

	fn libinput_udev_assign_seat(context: u64, name: u64) -> i32;
	fn libinput_get_fd(context: u64) -> std::os::fd::RawFd;
	fn libinput_dispatch(context: u64) -> i32;
	fn libinput_get_event(context: u64) -> Option<Event>;
	fn libinput_event_get_type(event: u64) -> i32;
	fn libinput_event_get_keyboard_event(event: u64) -> Option<EventKeyboard>;
	fn libinput_event_get_pointer_event(event: u64) -> Option<EventPointer>;
	fn libinput_event_keyboard_get_key(event_keyboard: u64) -> u32;
	fn libinput_event_keyboard_get_key_state(event_keyboard: u64) -> i32;
	fn libinput_event_pointer_get_dx_unaccelerated(event_pointer: u64) -> f64;
	fn libinput_event_pointer_get_dy_unaccelerated(event_pointer: u64) -> f64;
	fn libinput_event_pointer_get_button(event_pointer: u64) -> u32;
	fn libinput_event_pointer_get_button_state(event_pointer: u64) -> u32;
	fn libinput_event_pointer_get_scroll_value_v120(event_pointer: u64, axis: u32) -> f64;
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Context {
	ptr: std::ptr::NonNull<()>,
}

impl Context {
	pub fn create_from_udev(udev: udev::Instance) -> Option<Self> {
		extern "C" fn open(path: *const i8, flags: i32, _user_data: u64) -> i32 {
			unsafe { nix::libc::open(path, flags) }
		}

		extern "C" fn close(fd: std::os::fd::RawFd, _user_data: u64) {
			unsafe { nix::libc::close(fd) };
		}

		let interface = Box::leak(Box::new([open as u64, close as u64]));

		unsafe { libinput_udev_create_context(interface.as_ptr() as u64, 0, udev) }
	}

	pub fn assign(&self) -> i32 {
		unsafe { libinput_udev_assign_seat(self.ptr.as_ptr() as _, c"seat0".as_ptr() as u64) }
	}

	pub fn get_fd(&self) -> std::os::fd::RawFd {
		unsafe { libinput_get_fd(self.ptr.as_ptr() as _) }
	}

	pub fn dispatch(&self) -> i32 {
		unsafe { libinput_dispatch(self.ptr.as_ptr() as _) }
	}

	pub fn get_event(&self) -> Option<Event> {
		unsafe { libinput_get_event(self.ptr.as_ptr() as _) }
	}
}

#[repr(transparent)]
pub struct Event {
	ptr: std::ptr::NonNull<()>,
}

impl Event {
	pub fn get_type(&self) -> i32 {
		unsafe { libinput_event_get_type(self.ptr.as_ptr() as _) }
	}

	pub fn get_keyboard_event(&self) -> Option<EventKeyboard> {
		unsafe { libinput_event_get_keyboard_event(self.ptr.as_ptr() as _) }
	}

	pub fn get_pointer_event(&self) -> Option<EventPointer> {
		unsafe { libinput_event_get_pointer_event(self.ptr.as_ptr() as _) }
	}
}

#[repr(transparent)]
pub struct EventKeyboard {
	ptr: std::ptr::NonNull<()>,
}

impl EventKeyboard {
	pub fn get_key(&self) -> u32 {
		unsafe { libinput_event_keyboard_get_key(self.ptr.as_ptr() as _) }
	}

	pub fn get_key_state(&self) -> i32 {
		unsafe { libinput_event_keyboard_get_key_state(self.ptr.as_ptr() as _) }
	}
}

#[repr(transparent)]
pub struct EventPointer {
	ptr: std::ptr::NonNull<()>,
}

impl EventPointer {
	pub fn get_dx(&self) -> f64 {
		unsafe { libinput_event_pointer_get_dx_unaccelerated(self.ptr.as_ptr() as _) }
	}

	pub fn get_dy(&self) -> f64 {
		unsafe { libinput_event_pointer_get_dy_unaccelerated(self.ptr.as_ptr() as _) }
	}

	pub fn get_button(&self) -> u32 {
		unsafe { libinput_event_pointer_get_button(self.ptr.as_ptr() as _) }
	}

	pub fn get_button_state(&self) -> u32 {
		unsafe { libinput_event_pointer_get_button_state(self.ptr.as_ptr() as _) }
	}

	pub fn get_scroll_value_v120(&self, axis: u32) -> f64 {
		unsafe { libinput_event_pointer_get_scroll_value_v120(self.ptr.as_ptr() as _, axis) }
	}
}
