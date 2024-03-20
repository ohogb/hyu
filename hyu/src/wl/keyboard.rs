use std::{io::Seek, os::fd::AsRawFd};

use crate::{wl, Result};

pub struct Keyboard {
	object_id: u32,
	pub key_states: [bool; 0x100],
	serial: u32,
}

impl Keyboard {
	pub fn new(object_id: u32) -> Self {
		Self {
			object_id,
			key_states: [false; _],
			serial: 0,
		}
	}

	pub fn keymap(&mut self, client: &mut wl::Client) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:keymap
		let file = Box::leak(Box::new(std::fs::File::open("xkb")?));

		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 0,
			args: (1, file.stream_len()? as u32),
		})?;

		client.to_send_fds.push(file.as_raw_fd());

		Ok(())
	}

	pub fn enter(&mut self, client: &mut wl::Client, surface: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:enter
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 1,
			args: (self.serial(), surface, &[] as &[i32]),
		})?;

		self.modifiers(client)?;

		Ok(())
	}

	pub fn leave(&mut self, client: &mut wl::Client, surface: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:leave
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 2,
			args: (self.serial(), surface),
		})?;

		Ok(())
	}

	pub fn key(&mut self, client: &mut wl::Client, key: u32, state: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:key
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 3,
			args: (self.serial(), 100, key, state),
		})?;

		Ok(())
	}

	pub fn modifiers(&mut self, client: &mut wl::Client) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:modifiers
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 4,
			args: (self.serial(), 0, 0, 0, 0),
		})?;

		Ok(())
	}

	pub fn repeat_info(&mut self, client: &mut wl::Client, rate: i32, delay: i32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:repeat_info
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 5,
			args: (rate, delay),
		})?;

		Ok(())
	}

	fn serial(&mut self) -> u32 {
		let ret = self.serial;
		self.serial += 1;

		ret
	}
}

impl wl::Object for Keyboard {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_keyboard:request:release
			}
			_ => Err(format!("unknown op '{op}' in Keyboard"))?,
		}

		Ok(())
	}
}
