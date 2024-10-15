use crate::{wl, Client, Result};

pub struct Shm {
	object_id: wl::Id<Self>,
}

impl Shm {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}

	fn format(&self, client: &mut Client, format: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_shm:event:format
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: format,
		})
	}
}

impl wl::Object for Shm {
	fn handle(&mut self, client: &mut Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_shm:request:create_pool
				let (id, size): (wl::Id<wl::ShmPool>, u32) = wlm::decode::from_slice(params)?;
				let fd = client.received_fds.pop_front().unwrap();

				client.new_object(id, wl::ShmPool::new(id, fd, size)?);
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Shm"),
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

	fn bind(&self, client: &mut Client, object_id: u32, _version: u32) -> Result<()> {
		let shm = client.new_object(wl::Id::new(object_id), Self::new(wl::Id::new(object_id)));

		shm.format(client, 0)?;
		shm.format(client, 1)?;

		Ok(())
	}
}
