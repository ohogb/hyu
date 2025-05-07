use std::rc::Rc;

use crate::{Client, Connection, Point, Result, state::HwState, wl};

#[derive(Debug)]
pub struct DmabufAttributes {
	pub width: u32,
	pub height: u32,
	pub format: u32,
	pub modifier: u64,
	pub planes: Vec<Plane>,
}

#[derive(Debug)]
pub struct Plane {
	pub fd: std::os::fd::RawFd,
	pub offset: u32,
	pub stride: u32,
}

pub struct ZwpLinuxBufferParamsV1 {
	object_id: wl::Id<Self>,
	conn: Rc<Connection>,
	modifier: Option<u64>,
	planes: Vec<Plane>,
}

impl ZwpLinuxBufferParamsV1 {
	pub fn new(object_id: wl::Id<Self>, conn: Rc<Connection>) -> Self {
		Self {
			object_id,
			conn,
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

				assert!(!self.planes.is_empty());
				assert!(self.modifier.is_some());

				let modifier = self.modifier.unwrap();

				let (image, image_view) = hw_state.drm.vulkan.create_image_from_dmabuf(
					width as _,
					height as _,
					match format {
						0x34325241 | 0x34325258 => ash::vk::Format::B8G8R8A8_UNORM,
						0x34324241 => ash::vk::Format::R8G8B8A8_UNORM,
						_ => panic!("{}", format),
					},
					modifier,
					&self.planes,
				)?;

				let attributes = DmabufAttributes {
					width: width as _,
					height: height as _,
					format,
					modifier,
					planes: std::mem::take(&mut self.planes),
				};

				client.new_object(
					buffer_id,
					wl::Buffer::new(
						buffer_id,
						self.conn.clone(),
						wl::BufferBackingStorage::Dmabuf(wl::DmabufBackingStorage {
							size: Point(width, height),
							attributes,
							image,
							image_view,
							gbm_buffer_object: None,
						}),
					),
				);
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ZwpLinuxBufferParamsV1"),
		}

		Ok(())
	}
}
