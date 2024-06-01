use crate::{wl, Point, Result};

pub struct ZwpLinuxBufferParamsV1 {
	object_id: wl::Id<Self>,
	buffers: Vec<(std::os::fd::RawFd, u32, u32, u32, u32, u32)>,
}

impl ZwpLinuxBufferParamsV1 {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self {
			object_id,
			buffers: Vec::new(),
		}
	}
}

impl wl::Object for ZwpLinuxBufferParamsV1 {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_buffer_params_v1:request:destroy
				client.remove_object(self.object_id)?;
			}
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
				assert!(plane_idx == 0);

				self.buffers
					.push((fd, plane_idx, offset, stride, modifier_hi, modifier_lo));
			}
			3 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_buffer_params_v1:request:create_immed
				let (buffer_id, width, height, format, _flags): (
					wl::Id<wl::Buffer>,
					i32,
					i32,
					u32,
					u32,
				) = wlm::decode::from_slice(params)?;

				let mut attributes = Vec::new();

				attributes.push(0x3057);
				attributes.push(width);

				attributes.push(0x3056);
				attributes.push(height);

				attributes.push(0x3271);
				attributes.push(format as _);

				attributes.push(0x30D2);
				attributes.push(1);

				let buffer = self.buffers.first().unwrap();

				attributes.push(0x3272);
				attributes.push(buffer.0);

				attributes.push(0x3273);
				attributes.push(buffer.2 as _);

				attributes.push(0x3274);
				attributes.push(buffer.3 as _);

				attributes.push(0x3443);
				attributes.push(buffer.5 as _);

				attributes.push(0x3444);
				attributes.push(buffer.4 as _);

				attributes.push(0x3038);

				let image = crate::backend::gl::egl_wrapper::create_image(0x3270, &attributes);

				eprintln!("image: {image:X}",);

				client.new_object(
					buffer_id,
					wl::Buffer::new(
						buffer_id,
						Point(width, height),
						wl::BufferStorage::Dmabuf { image },
					),
				);
			}
			_ => Err(format!("unknown op '{op}' in ZwpLinuxBufferParamsV1"))?,
		}

		Ok(())
	}
}
