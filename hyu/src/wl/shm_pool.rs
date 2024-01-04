use crate::{wl, Result};

pub struct ShmPool {}

impl ShmPool {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for ShmPool {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				let (id, _offset, _width, _height, _stride, _format): (
					u32,
					u32,
					u32,
					u32,
					u32,
					u32,
				) = wlm::decode::from_slice(&params)?;

				client.push_client_object(id, wl::Buffer::new());
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:destroy
			}
			_ => Err(format!("unknown op '{op}' in ShmPool"))?,
		}

		Ok(())
	}
}
