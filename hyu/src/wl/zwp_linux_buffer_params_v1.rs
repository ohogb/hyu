use crate::{wl, Result};

pub struct ZwpLinuxBufferParamsV1 {
	object_id: wl::Id<Self>,
}

impl ZwpLinuxBufferParamsV1 {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}
}

impl wl::Object for ZwpLinuxBufferParamsV1 {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			1 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_buffer_params_v1:request:add
				let (plane_idx, offset, stride, modifier_hi, modifier_lo): (
					u32,
					u32,
					u32,
					u32,
					u32,
				) = wlm::decode::from_slice(params)?;

				let fd = client.received_fds.pop_front().unwrap();
			}
			_ => Err(format!("unknown op '{op}' in ZwpLinuxBufferParamsV1"))?,
		}

		Ok(())
	}
}
