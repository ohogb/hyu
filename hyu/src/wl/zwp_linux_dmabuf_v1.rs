use crate::{wl, Result};

pub struct ZwpLinuxDmabufV1 {
	object_id: wl::Id<Self>,
}

impl ZwpLinuxDmabufV1 {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}
}

impl wl::Object for ZwpLinuxDmabufV1 {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			2 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_dmabuf_v1:request:get_default_feedback
				let id: wl::Id<wl::ZwpLinuxDmabufFeedbackV1> = wlm::decode::from_slice(params)?;
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
