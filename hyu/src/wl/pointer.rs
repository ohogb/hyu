use crate::{wl, Result};

pub struct Pointer {
	object_id: u32,
	serial: u32,
}

impl Pointer {
	pub fn new(object_id: u32) -> Self {
		Self {
			object_id,
			serial: 0,
		}
	}

	pub fn enter(&mut self, client: &mut wl::Client, surface: u32, x: i32, y: i32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:enter
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 0,
			args: (
				self.serial(),
				surface,
				fixed::types::I24F8::from_num(x),
				fixed::types::I24F8::from_num(y),
			),
		})?;

		Ok(())
	}

	pub fn leave(&mut self, client: &mut wl::Client, surface: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:leave
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 1,
			args: (self.serial(), surface),
		})?;

		Ok(())
	}

	pub fn motion(&mut self, client: &mut wl::Client, x: i32, y: i32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:motion
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 2,
			args: (
				self.serial(),
				fixed::types::I24F8::from_num(x),
				fixed::types::I24F8::from_num(y),
			),
		})?;

		Ok(())
	}

	pub fn button(&mut self, client: &mut wl::Client, button: u32, state: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:button
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 3,
			args: (self.serial(), 0, button, state),
		})?;

		Ok(())
	}

	fn serial(&mut self) -> u32 {
		let ret = self.serial;
		self.serial += 1;

		ret
	}
}

impl wl::Object for Pointer {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_pointer:request:set_cursor
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_pointer:request:release
			}
			_ => Err(format!("unknown op '{op}' in Pointer"))?,
		}

		Ok(())
	}
}
