use crate::{wl, Result};

pub struct ShmPool {
	object_id: u32,
	fd: std::os::fd::RawFd,
	size: u32,
}

impl ShmPool {
	pub fn new(object_id: u32, fd: std::os::fd::RawFd, size: u32) -> Self {
		Self {
			object_id,
			fd,
			size,
		}
	}
}

impl wl::Object for ShmPool {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:create_buffer
				let (id, offset, width, height, stride, format): (u32, i32, i32, i32, i32, u32) =
					wlm::decode::from_slice(&params)?;

				client.push_client_object(
					id,
					wl::Buffer::new(
						id, self.fd, self.size, offset, width, height, stride, format,
					),
				);
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_shm_pool:request:destroy
			}
			_ => Err(format!("unknown op '{op}' in ShmPool"))?,
		}

		Ok(())
	}
}
