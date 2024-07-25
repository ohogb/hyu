use crate::{wl, Result};

pub struct Keyboard {
	object_id: wl::Id<Self>,
	seat_id: wl::Id<wl::Seat>,
	pub key_states: [bool; 0x100],
	keymap: (std::os::fd::RawFd, u64),
}

impl Keyboard {
	pub fn new(
		object_id: wl::Id<Self>,
		seat_id: wl::Id<wl::Seat>,
		keymap: (std::os::fd::RawFd, u64),
	) -> Self {
		Self {
			object_id,
			seat_id,
			key_states: [false; _],
			keymap,
		}
	}

	pub fn keymap(&mut self, client: &mut wl::Client) -> Result<()> {
		let (fd, size) = self.keymap;

		// https://wayland.app/protocols/wayland#wl_keyboard:event:keymap
		client.to_send_fds.push(fd);

		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (1, size as u32),
		})?;

		Ok(())
	}

	pub fn enter(&mut self, client: &mut wl::Client, surface: wl::Id<wl::Surface>) -> Result<()> {
		let display = client.get_object_mut(wl::Id::<wl::Display>::new(1))?;

		// https://wayland.app/protocols/wayland#wl_keyboard:event:enter
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: (display.new_serial(), surface, &[] as &[i32]),
		})?;

		self.modifiers(client, 0)
	}

	pub fn leave(&mut self, client: &mut wl::Client, surface: wl::Id<wl::Surface>) -> Result<()> {
		let display = client.get_object_mut(wl::Id::<wl::Display>::new(1))?;

		// https://wayland.app/protocols/wayland#wl_keyboard:event:leave
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 2,
			args: (display.new_serial(), surface),
		})
	}

	pub fn key(&mut self, client: &mut wl::Client, key: u32, state: u32) -> Result<()> {
		let display = client.get_object_mut(wl::Id::<wl::Display>::new(1))?;

		// https://wayland.app/protocols/wayland#wl_keyboard:event:key
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 3,
			args: (display.new_serial(), 100, key, state),
		})
	}

	pub fn modifiers(&mut self, client: &mut wl::Client, depressed: u32) -> Result<()> {
		let display = client.get_object_mut(wl::Id::<wl::Display>::new(1))?;

		// https://wayland.app/protocols/wayland#wl_keyboard:event:modifiers
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 4,
			args: (display.new_serial(), depressed, 0, 0, 0),
		})
	}

	pub fn repeat_info(&mut self, client: &mut wl::Client, rate: i32, delay: i32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:repeat_info
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 5,
			args: (rate, delay),
		})
	}
}

impl wl::Object for Keyboard {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_keyboard:request:release
			}
			_ => Err(format!("unknown op '{op}' in Keyboard"))?,
		}

		Ok(())
	}
}
