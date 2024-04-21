use std::{io::Seek, os::fd::AsRawFd};

use crate::{wl, Result};

pub struct ZwpLinuxDmabufFeedbackV1 {
	object_id: wl::Id<Self>,
}

impl ZwpLinuxDmabufFeedbackV1 {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}

	pub fn done(&self, client: &mut wl::Client) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:done
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (),
		})
	}

	pub fn format_table(&self, client: &mut wl::Client) -> Result<()> {
		let file = Box::leak(Box::new(std::fs::File::open("formats")?));

		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:format_table
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: file.stream_len()? as u32,
		})?;

		client.to_send_fds.push(file.as_raw_fd());

		Ok(())
	}

	pub fn main_device(&self, client: &mut wl::Client, device: &[u32]) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:main_device
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 2,
			args: device,
		})
	}

	pub fn tranche_done(&self, client: &mut wl::Client) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:tranche_done
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 3,
			args: (),
		})
	}
}

impl wl::Object for ZwpLinuxDmabufFeedbackV1 {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:request:destroy
				client.remove_object(self.object_id)?;
			}
			_ => Err(format!("unknown op '{op}' in ZwpLinuxDmabufFeedbackV1"))?,
		}

		Ok(())
	}
}
