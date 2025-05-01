use std::{
	io::{Seek as _, Write as _},
	os::fd::IntoRawFd as _,
};

use crate::{Client, Config, Result, state::HwState, wl};

struct Format {
	format: u32,
	modifier: u64,
}

const FORMATS: [Format; 3] = [
	// AR24
	Format {
		format: 0x34325241,
		modifier: 0x0,
	},
	// Format {
	// 	format: 0x34325241,
	// 	modifier: 0x20000002096BB03,
	// },
	// XR24
	Format {
		format: 0x34325258,
		modifier: 0x0,
	},
	// Format {
	// 	format: 0x34325258,
	// 	modifier: 0x20000002096BB03,
	// },
	// AB24
	Format {
		format: 0x34324241,
		modifier: 0x0,
	},
	// Format {
	// 	format: 0x34324241,
	// 	modifier: 0x20000002096BB03,
	// },
];

pub struct ZwpLinuxDmabufV1 {
	object_id: wl::Id<Self>,
	formats: (std::os::fd::RawFd, u64),
	config: &'static Config,
}

impl ZwpLinuxDmabufV1 {
	pub fn new(object_id: wl::Id<Self>, config: &'static Config) -> Result<Self> {
		let (fd, path) = nix::unistd::mkstemp("/tmp/temp_XXXXXX")?;
		nix::unistd::unlink(&path)?;

		let mut file = std::fs::File::from(fd);

		for format in FORMATS {
			file.write_all(&u64::to_ne_bytes(format.format as _))?;
			file.write_all(&u64::to_ne_bytes(format.modifier))?;
		}

		let size = file.stream_len()?;
		let fd = file.into_raw_fd();

		Ok(Self {
			object_id,
			formats: (fd, size),
			config,
		})
	}

	pub fn format(&self, client: &mut Client, format: u32) -> Result<()> {
		// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_v1:event:format
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: format,
		})
	}

	pub fn modifier(
		&self,
		client: &mut Client,
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
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_v1:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
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

				let dev = nix::sys::stat::stat(&self.config.card)?.st_rdev;
				feedback.main_device(client, &[dev])?;

				feedback.tranche_target_device(client, &[dev])?;
				feedback.tranche_flags(client, 1 << 0)?;
				feedback
					.tranche_formats(client, &(0..(FORMATS.len() as u16)).collect::<Vec<_>>())?;
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

				let dev = nix::sys::stat::stat(&self.config.card)?.st_rdev;
				feedback.main_device(client, &[dev])?;

				feedback.tranche_target_device(client, &[dev])?;
				feedback.tranche_flags(client, 1 << 0)?;
				feedback
					.tranche_formats(client, &(0..(FORMATS.len() as u16)).collect::<Vec<_>>())?;
				feedback.tranche_done(client)?;

				feedback.done(client)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ZwpLinuxDmabufV1"),
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

	fn bind(&self, client: &mut Client, object_id: u32, version: u32) -> Result<()> {
		let id = wl::Id::new(object_id);
		let object = client.new_object(id, Self::new(id, self.config)?);

		assert!(version >= 3);

		if version == 3 {
			for format in FORMATS {
				object.modifier(
					client,
					format.format,
					((format.modifier >> 32) & 0xFFFF_FFFF) as u32,
					(format.modifier & 0xFFFF_FFFF) as u32,
				)?;
			}
		}

		Ok(())
	}
}
