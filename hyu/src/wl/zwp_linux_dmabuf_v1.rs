use crate::{wl, Result};

pub struct ZwpLinuxDmabufV1 {
	object_id: wl::Id<Self>,
}

impl ZwpLinuxDmabufV1 {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
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
				client.new_object(id, wl::ZwpLinuxDmabufFeedbackV1::new(id));
			}
			3 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_v1:request:get_surface_feedback
				let (id, surface): (wl::Id<wl::ZwpLinuxDmabufFeedbackV1>, wl::Id<wl::Surface>) =
					wlm::decode::from_slice(params)?;

				client.new_object(id, wl::ZwpLinuxDmabufFeedbackV1::new(id));
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
		client.new_object(id, Self::new(id));

		Ok(())
	}
}
