use color_eyre::eyre::OptionExt as _;

use crate::{Client, Point, Result, state::HwState, wl};

pub struct Plane {
	pub fd: std::os::fd::RawFd,
	pub offset: u32,
	pub stride: u32,
}

pub struct ZwpLinuxBufferParamsV1 {
	object_id: wl::Id<Self>,
	modifier: Option<u64>,
	planes: Vec<Plane>,
}

impl ZwpLinuxBufferParamsV1 {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self {
			object_id,
			modifier: None,
			planes: Vec::new(),
		}
	}
}

impl wl::Object for ZwpLinuxBufferParamsV1 {
	fn handle(
		&mut self,
		client: &mut Client,
		hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_buffer_params_v1:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			1 => {
				// https://wayland.app/protocols/linux-dmabuf-v1#zwp_linux_buffer_params_v1:request:add
				let (_plane_idx, offset, stride, modifier_hi, modifier_lo): (
					u32,
					u32,
					u32,
					u32,
					u32,
				) = wlm::decode::from_slice(params)?;

				let fd = client.received_fds.pop_front().unwrap();

				let modifier = ((modifier_hi as u64) << 32) | modifier_lo as u64;

				if let Some(other_plane_mod) = &self.modifier {
					if modifier != *other_plane_mod {
						panic!();
					}
				} else {
					self.modifier = Some(modifier);
				}

				self.planes.push(Plane { fd, offset, stride });
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

				// let mut attributes = Vec::new();
				//
				// attributes.push(0x3057);
				// attributes.push(width);
				//
				// attributes.push(0x3056);
				// attributes.push(height);
				//
				// attributes.push(0x3271);
				// attributes.push(format as _);
				//
				// attributes.push(0x30D2);
				// attributes.push(1);
				//
				// for (index, buffer) in self.buffers.iter().enumerate() {
				// 	let index = index as i32;
				//
				// 	attributes.push(0x3272 + index * 3);
				// 	attributes.push(buffer.0);
				//
				// 	attributes.push(0x3273 + index * 3);
				// 	attributes.push(buffer.2 as _);
				//
				// 	attributes.push(0x3274 + index * 3);
				// 	attributes.push(buffer.3 as _);
				//
				// 	attributes.push(0x3443 + index * 2);
				// 	attributes.push(buffer.5 as _);
				//
				// 	attributes.push(0x3444 + index * 2);
				// 	attributes.push(buffer.4 as _);
				// }
				//
				// attributes.push(0x3038);
				//
				// let image = crate::egl::DISPLAY
				// 	.create_image(0x3270, &attributes)
				// 	.ok_or_eyre("failed to create egl image")?;
				assert!(!self.planes.is_empty());
				assert!(self.modifier.is_some());

				let (image, image_view) = hw_state.drm.vulkan.create_image_from_dmabuf(
					width as _,
					height as _,
					match format {
						0x34325241 | 0x34325258 => ash::vk::Format::B8G8R8A8_UNORM,
						0x34324241 => ash::vk::Format::R8G8B8A8_UNORM,
						_ => panic!("{}", format),
					},
					self.modifier.unwrap(),
					&self.planes,
				)?;

				client.new_object(
					buffer_id,
					wl::Buffer::new(buffer_id, Point(width, height), wl::BufferStorage::Dmabuf {
						image,
						image_view,
					}),
				);

				// for &(fd, ..) in &self.buffers {
				// 	nix::unistd::close(fd)?;
				// }
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ZwpLinuxBufferParamsV1"),
		}

		Ok(())
	}
}
