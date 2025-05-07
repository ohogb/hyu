use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct Output {
	pub object_id: wl::Id<Self>,
	conn: Rc<Connection>,
}

impl Output {
	pub fn new(object_id: wl::Id<Self>, conn: Rc<Connection>) -> Self {
		Self { object_id, conn }
	}

	fn geometry(
		&self,
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
		self.conn.send_message(wlm::Message {
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

	fn mode(&self, flags: u32, width: i32, height: i32, refresh: i32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_output:event:mode
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: (flags, width, height, refresh),
		})
	}

	pub fn done(&self) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_output:event:done
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 2,
			args: (),
		})
	}

	fn scale(&self, factor: i32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_output:event:scale
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 3,
			args: factor,
		})
	}
}

impl wl::Object for Output {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		_params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_output:request:release
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Output"),
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

	fn bind(&self, client: &mut Client, object_id: u32, _version: u32) -> Result<()> {
		let output = client.new_object(
			wl::Id::new(object_id),
			Self::new(wl::Id::new(object_id), self.conn.clone()),
		);

		output.geometry(0, 0, 600, 340, 0, "AUS", "ROG XG27AQM", 0)?;
		output.mode(3, 2560, 1440, 270000)?;
		output.scale(1)?;
		output.done()?;

		Ok(())
	}
}
