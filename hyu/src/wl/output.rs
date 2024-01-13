use crate::{wl, Result};

#[derive(Debug)]
pub struct Output {}

impl Output {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Output {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_output:request:release
			}
			_ => Err(format!("unknown op '{op}' in Output"))?,
		}

		Ok(())
	}
}

impl wl::Global for Output {
	fn get_name(&self) -> &'static str {
		"wl_output"
	}

	fn get_version(&self) -> u32 {
		3
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		client.push_client_object(object_id, Self::new());

		client.send_message(wlm::Message {
			object_id,
			op: 0,
			args: (0u32, 0u32, 600u32, 340u32, 0u32, "AUS", "ROG XG27AQM", 0u32),
		})?;

		client.send_message(wlm::Message {
			object_id,
			op: 1,
			args: (3u32, 2560u32, 1440u32, 270000u32),
		})?;

		client.send_message(wlm::Message {
			object_id,
			op: 3,
			args: 1u32,
		})?;

		client.send_message(wlm::Message {
			object_id,
			op: 2,
			args: (),
		})?;

		Ok(())
	}
}
