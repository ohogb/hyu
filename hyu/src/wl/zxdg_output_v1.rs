use crate::{wl, Client, Result};

pub struct ZxdgOutputV1 {
	object_id: wl::Id<Self>,
	#[expect(unused)]
	output_id: wl::Id<wl::Output>,
}

impl ZxdgOutputV1 {
	pub fn new(object_id: wl::Id<Self>, output_id: wl::Id<wl::Output>) -> Self {
		Self {
			object_id,
			output_id,
		}
	}

	pub fn logical_position(&self, client: &mut wl::Client, x: i32, y: i32) -> Result<()> {
		// https://wayland.app/protocols/xdg-output-unstable-v1#zxdg_output_v1:event:logical_position
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (x, y),
		})
	}

	pub fn logical_size(&self, client: &mut wl::Client, width: i32, height: i32) -> Result<()> {
		// https://wayland.app/protocols/xdg-output-unstable-v1#zxdg_output_v1:event:logical_size
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: (width, height),
		})
	}

	pub fn done(&self, client: &mut wl::Client) -> Result<()> {
		// https://wayland.app/protocols/xdg-output-unstable-v1#zxdg_output_v1:event:done
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 2,
			args: (),
		})
	}
}

impl wl::Object for ZxdgOutputV1 {
	fn handle(&mut self, client: &mut Client, op: u16, _params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-output-unstable-v1#zxdg_output_v1:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ZxdgOutputV1"),
		}

		Ok(())
	}
}
