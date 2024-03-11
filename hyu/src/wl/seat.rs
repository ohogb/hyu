use crate::{wl, Result};

#[derive(Debug)]
pub struct Seat {}

impl Seat {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Seat {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:get_pointer
				let id: u32 = wlm::decode::from_slice(&params)?;
				client.push_client_object(id, wl::Pointer::new(id));
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:get_keyboard
				let id: u32 = wlm::decode::from_slice(&params)?;

				let mut keyboard = wl::Keyboard::new(id);
				keyboard.repeat_info(client, 500, 500)?;

				client.push_client_object(id, keyboard);
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
		client.push_client_object(object_id, Self::new());

		client.send_message(wlm::Message {
			object_id,
			op: 0,
			args: 3u32,
		})?;

		Ok(())
	}
}
