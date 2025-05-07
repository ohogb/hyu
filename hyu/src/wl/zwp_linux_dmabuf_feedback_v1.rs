use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct ZwpLinuxDmabufFeedbackV1 {
	object_id: wl::Id<Self>,
	conn: Rc<Connection>,
	formats: (std::os::fd::RawFd, u64),
}

impl ZwpLinuxDmabufFeedbackV1 {
	pub fn new(
		object_id: wl::Id<Self>,
		conn: Rc<Connection>,
		formats: (std::os::fd::RawFd, u64),
	) -> Self {
		Self {
			object_id,
			conn,
			formats,
		}
	}

	pub fn done(&self) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:done
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (),
		})
	}

	pub fn format_table(&self) -> Result<()> {
		let (fd, size) = self.formats;

		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:format_table
		self.conn.send_message_with_fd(
			wlm::Message {
				object_id: *self.object_id,
				op: 1,
				args: size as u32,
			},
			fd,
		)?;

		Ok(())
	}

	pub fn main_device(&self, device: &[nix::libc::dev_t]) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:main_device
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 2,
			args: device,
		})
	}

	pub fn tranche_done(&self) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:tranche_done
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 3,
			args: (),
		})
	}

	pub fn tranche_target_device(&self, device: &[nix::libc::dev_t]) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:tranche_target_device
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 4,
			args: device,
		})
	}

	pub fn tranche_formats(&self, indices: &[u16]) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:tranche_formats
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 5,
			args: indices,
		})
	}

	pub fn tranche_flags(&self, flags: u32) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:tranche_flags
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 6,
			args: flags,
		})
	}
}

impl wl::Object for ZwpLinuxDmabufFeedbackV1 {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		_params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ZwpLinuxDmabufFeedbackV1"),
		}

		Ok(())
	}
}
