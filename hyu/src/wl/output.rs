use crate::{wl, Result};

pub struct Output {
	object_id: wl::Id<Self>,
}

impl Output {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}

	fn geometry(
		&self,
		client: &mut wl::Client,
		x: i32,
		y: i32,
		physical_width: i32,
		physical_height: i32,
		subpixel: i32,
		make: &str,
		model: &str,
		transform: i32,
	) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_output:event:geometry
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (
				x,
				y,
				physical_width,
				physical_height,
				subpixel,
				make,
				model,
				transform,
			),
		})
	}
}

impl wl::Object for Output {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_output:request:release
			}
			_ => Err(format!("unknown op '{op}' in Output"))?,
		}

		Ok(())
	}
}

impl wl::Global for Output {
	fn get_name(&self) -> &'static str {
		"wl_output"
	}

	fn get_version(&self) -> u32 {
		3
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		let output = client.new_object(wl::Id::new(object_id), Self::new(wl::Id::new(object_id)));

		output.geometry(client, 0, 0, 600, 340, 0, "AUS", "ROG XG27AQM", 0)?;

		client.send_message(wlm::Message {
			object_id,
			op: 1,
			args: (3u32, 2560u32, 1440u32, 270000u32),
		})?;

		client.send_message(wlm::Message {
			object_id,
			op: 3,
			args: 1u32,
		})?;

		client.send_message(wlm::Message {
			object_id,
			op: 2,
			args: (),
		})?;

		Ok(())
	}
}
