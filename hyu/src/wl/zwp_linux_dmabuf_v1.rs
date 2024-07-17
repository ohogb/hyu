use std::{
	io::{Seek as _, Write as _},
	os::fd::{FromRawFd as _, IntoRawFd as _},
};

use crate::{wl, Result};

pub struct ZwpLinuxDmabufV1 {
	object_id: wl::Id<Self>,
	formats: (std::os::fd::RawFd, u64),
}

impl ZwpLinuxDmabufV1 {
	pub fn new(object_id: wl::Id<Self>) -> Result<Self> {
		let (fd, path) = nix::unistd::mkstemp("/tmp/temp_XXXXXX")?;
		nix::unistd::unlink(&path)?;

		let mut file = unsafe { std::fs::File::from_raw_fd(fd) };

		file.write_all(&u64::to_ne_bytes(0x34325241))?;
		file.write_all(&u64::to_ne_bytes(0x20000002096BB03))?;

		file.write_all(&u64::to_ne_bytes(0x34325241))?;
		file.write_all(&u64::to_ne_bytes(0x0))?;

		file.write_all(&u64::to_ne_bytes(0x34325241))?;
		file.write_all(&u64::to_ne_bytes(0xFFFFFFFFFFFFFF))?;

		file.write_all(&u64::to_ne_bytes(0x34325258))?;
		file.write_all(&u64::to_ne_bytes(0x20000002096BB03))?;

		file.write_all(&u64::to_ne_bytes(0x34325258))?;
		file.write_all(&u64::to_ne_bytes(0x0))?;

		file.write_all(&u64::to_ne_bytes(0x34325258))?;
		file.write_all(&u64::to_ne_bytes(0xFFFFFFFFFFFFFF))?;

		let size = file.stream_len()?;
		let fd = file.into_raw_fd();

		Ok(Self {
			object_id,
			formats: (fd, size),
		})
	}

	pub fn format(&self, client: &mut wl::Client, format: u32) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_v1:event:format
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: format,
		})
	}

	pub fn modifier(
		&self,
		client: &mut wl::Client,
		format: u32,
		modifier_hi: u32,
		modifier_lo: u32,
	) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_v1:event:modifier
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: (format, modifier_hi, modifier_lo),
		})
	}
}

impl wl::Object for ZwpLinuxDmabufV1 {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_v1:request:destroy
				client.remove_object(self.object_id)?;
			}
			1 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_v1:request:create_params
				let id: wl::Id<wl::ZwpLinuxBufferParamsV1> = wlm::decode::from_slice(params)?;
				client.new_object(id, wl::ZwpLinuxBufferParamsV1::new(id));
			}
			2 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_v1:request:get_default_feedback
				let id: wl::Id<wl::ZwpLinuxDmabufFeedbackV1> = wlm::decode::from_slice(params)?;
				let feedback =
					client.new_object(id, wl::ZwpLinuxDmabufFeedbackV1::new(id, self.formats));

				feedback.format_table(client)?;

				let dev = nix::sys::stat::stat("/dev/dri/card1")?.st_rdev;
				feedback.main_device(client, &[dev])?;

				feedback.tranche_target_device(client, &[dev])?;
				feedback.tranche_flags(client, 0)?;
				feedback.tranche_formats(client, &[0, 1, 2, 3, 4, 5])?;
				feedback.tranche_done(client)?;

				feedback.done(client)?;
			}
			3 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_v1:request:get_surface_feedback
				let (id, _surface): (wl::Id<wl::ZwpLinuxDmabufFeedbackV1>, wl::Id<wl::Surface>) =
					wlm::decode::from_slice(params)?;

				let feedback =
					client.new_object(id, wl::ZwpLinuxDmabufFeedbackV1::new(id, self.formats));

				feedback.format_table(client)?;

				let dev = nix::sys::stat::stat("/dev/dri/card1")?.st_rdev;
				feedback.main_device(client, &[dev])?;

				feedback.tranche_target_device(client, &[dev])?;
				feedback.tranche_flags(client, 0)?;
				feedback.tranche_formats(client, &[0, 1, 2, 3, 4, 5])?;
				feedback.tranche_done(client)?;

				feedback.done(client)?;
			}
			_ => Err(format!("unknown op '{op}' in ZwpLinuxDmabufV1"))?,
		}

		Ok(())
	}
}

impl wl::Global for ZwpLinuxDmabufV1 {
	fn get_name(&self) -> &'static str {
		"zwp_linux_dmabuf_v1"
	}

	fn get_version(&self) -> u32 {
		5
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		let id = wl::Id::new(object_id);
		client.new_object(id, Self::new(id)?);

		/*object.format(client, 0x34325241)?;
		object.modifier(client, 0x34325241, 0, 0)?;
		object.modifier(client, 0x34325241, 0x2000000, 0x2096bb03)?;
		object.modifier(client, 0x34325241, 0xFFFFFF, 0xFFFFFFFF)?;*/

		Ok(())
	}
}
