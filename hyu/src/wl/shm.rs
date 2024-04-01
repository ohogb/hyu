use crate::{wl, Result};

#[derive(Debug, Clone)]
pub struct Shm {}

impl Shm {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Shm {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_shm:request:create_pool
				let (id, size): (wl::Id<wl::ShmPool>, u32) = wlm::decode::from_slice(params)?;
				let fd = client.received_fds.pop_front().unwrap();

				client.new_object(id, wl::ShmPool::new(id, fd, size)?);
			}
			_ => Err(format!("unknown op '{op}' in Shm"))?,
		}

		Ok(())
	}
}

impl wl::Global for Shm {
	fn get_name(&self) -> &'static str {
		"wl_shm"
	}

	fn get_version(&self) -> u32 {
		1
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		client.new_object(wl::Id::new(object_id), Self::new());

		client.send_message(wlm::Message {
			object_id,
			op: 0,
			args: 0u32,
		})?;

		client.send_message(wlm::Message {
			object_id,
			op: 0,
			args: 1u32,
		})?;

		Ok(())
	}
}
