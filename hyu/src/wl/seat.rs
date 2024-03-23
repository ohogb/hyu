use crate::{wl, Result};

#[derive(Debug)]
pub struct Seat {
	object_id: u32,
	serial: u32,
}

impl Seat {
	pub fn new(object_id: u32) -> Self {
		Self {
			object_id,
			serial: 0,
		}
	}

	pub fn serial(&mut self) -> u32 {
		let ret = self.serial;
		self.serial += 1;

		ret
	}
}

impl wl::Object for Seat {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:get_pointer
				let id: u32 = wlm::decode::from_slice(&params)?;
				client.queue_new_object(id, wl::Pointer::new(id, self.object_id));
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:get_keyboard
				let id: u32 = wlm::decode::from_slice(&params)?;

				let mut keyboard = wl::Keyboard::new(id, self.object_id);
				keyboard.keymap(client)?;
				keyboard.repeat_info(client, 500, 500)?;

				client.queue_new_object(id, keyboard);
			}
			3 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:release
			}
			_ => Err(format!("unknown op '{op}' in Seat"))?,
		}

		Ok(())
	}
}

impl wl::Global for Seat {
	fn get_name(&self) -> &'static str {
		"wl_seat"
	}

	fn get_version(&self) -> u32 {
		7
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		client.queue_new_object(object_id, Self::new(object_id));

		client.send_message(wlm::Message {
			object_id,
			op: 0,
			args: 3u32,
		})?;

		Ok(())
	}
}
