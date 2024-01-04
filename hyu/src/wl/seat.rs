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
				let id: u32 = wlm::decode::from_slice(&params)?;
				client.push_client_object(id, wl::Pointer::new());
			}
			1 => {
				let id: u32 = wlm::decode::from_slice(&params)?;
				client.push_client_object(id, wl::Keyboard::new());
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

		let mut buf = Vec::new();

		buf.extend(object_id.to_ne_bytes());
		buf.extend(0u16.to_ne_bytes());

		let arg = wlm::encode::to_vec(&3u32).unwrap();

		buf.extend((8u16 + arg.len() as u16).to_ne_bytes());
		buf.extend(arg);

		client.get_state().buffer.0.extend(buf);

		Ok(())
	}
}
