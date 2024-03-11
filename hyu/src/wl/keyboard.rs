use crate::{wl, Result};

pub struct Keyboard {
	object_id: u32,
}

impl Keyboard {
	pub fn new(object_id: u32) -> Self {
		Self { object_id }
	}

	pub fn keymap(&mut self, client: &mut wl::Client) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:keymap
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 0,
			args: (0, 0, 0),
		})?;

		Ok(())
	}

	pub fn enter(&mut self, client: &mut wl::Client, surface: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:enter
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 1,
			args: (0, surface, &[] as &[i32]),
		})?;

		self.modifiers(client)?;

		Ok(())
	}

	pub fn leave(&mut self, client: &mut wl::Client, surface: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:leave
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 2,
			args: (0, surface),
		})?;

		Ok(())
	}

	pub fn key(&mut self, client: &mut wl::Client, key: u32, state: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:key
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 3,
			args: (1, 100, key, state),
		})?;

		Ok(())
	}

	pub fn modifiers(&mut self, client: &mut wl::Client) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_keyboard:event:modifiers
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 4,
			args: (0, 0, 0, 0, 0),
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
