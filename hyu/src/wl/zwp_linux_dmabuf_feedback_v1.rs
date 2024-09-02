use crate::{wl, Result};

pub struct ZwpLinuxDmabufFeedbackV1 {
	object_id: wl::Id<Self>,
	formats: (std::os::fd::RawFd, u64),
}

impl ZwpLinuxDmabufFeedbackV1 {
	pub fn new(object_id: wl::Id<Self>, formats: (std::os::fd::RawFd, u64)) -> Self {
		Self { object_id, formats }
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
		let (fd, size) = self.formats;

		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:format_table
		client.to_send_fds.push(fd);

		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: size as u32,
		})?;

		Ok(())
	}

	pub fn main_device(&self, client: &mut wl::Client, device: &[nix::libc::dev_t]) -> Result<()> {
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

	pub fn tranche_target_device(
		&self,
		client: &mut wl::Client,
		device: &[nix::libc::dev_t],
	) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:tranche_target_device
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 4,
			args: device,
		})
	}

	pub fn tranche_formats(&self, client: &mut wl::Client, indices: &[u16]) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:tranche_formats
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 5,
			args: indices,
		})
	}

	pub fn tranche_flags(&self, client: &mut wl::Client, flags: u32) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:event:tranche_flags
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 6,
			args: flags,
		})
	}
}

impl wl::Object for ZwpLinuxDmabufFeedbackV1 {
	fn handle(&mut self, client: &mut wl::Client, op: u16, _params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_feedback_v1:request:destroy
				client.remove_object(self.object_id)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ZwpLinuxDmabufFeedbackV1"),
		}

		Ok(())
	}
}
