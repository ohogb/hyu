use crate::{Client, Point, Result, wl};

pub struct Pointer {
	object_id: wl::Id<Self>,
	#[expect(dead_code)]
	seat_id: wl::Id<wl::Seat>,
	pub should_hide_cursor: bool,
}

impl Pointer {
	pub fn new(object_id: wl::Id<Self>, seat_id: wl::Id<wl::Seat>) -> Self {
		Self {
			object_id,
			seat_id,
			should_hide_cursor: false,
		}
	}

	pub fn enter(
		&mut self,
		client: &mut Client,
		serial: u32,
		surface: wl::Id<wl::Surface>,
		position: Point,
	) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:enter
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (
				serial,
				surface,
				fixed::types::I24F8::from_num(position.0),
				fixed::types::I24F8::from_num(position.1),
			),
		})
	}

	pub fn leave(
		&mut self,
		client: &mut Client,
		serial: u32,
		surface: wl::Id<wl::Surface>,
	) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:leave
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: (serial, surface),
		})
	}

	pub fn motion(&mut self, client: &mut Client, position: Point) -> Result<()> {
		let display = client.get_object(wl::Id::<wl::Display>::new(1))?;
		let time = display.get_time().as_millis();

		// https://wayland.app/protocols/wayland#wl_pointer:event:motion
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 2,
			args: (
				time as u32,
				fixed::types::I24F8::from_num(position.0),
				fixed::types::I24F8::from_num(position.1),
			),
		})
	}

	pub fn button(
		&mut self,
		client: &mut Client,
		serial: u32,
		button: u32,
		state: u32,
	) -> Result<()> {
		let display = client.get_object_mut(wl::Id::<wl::Display>::new(1))?;
		let time = display.get_time().as_millis();

		// https://wayland.app/protocols/wayland#wl_pointer:event:button
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 3,
			args: (serial, time as u32, button, state),
		})
	}

	pub fn axis(&mut self, client: &mut Client, axis: u32, value: f64) -> Result<()> {
		let display = client.get_object_mut(wl::Id::<wl::Display>::new(1))?;
		let time = display.get_time().as_millis();

		// https://wayland.app/protocols/wayland#wl_pointer:event:axis
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 4,
			args: (time as u32, axis, fixed::types::I24F8::from_num(value)),
		})
	}

	pub fn frame(&mut self, client: &mut Client) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:frame
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 5,
			args: (),
		})
	}

	pub fn axis_source(&mut self, client: &mut Client, axis_source: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:axis_source
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 6,
			args: axis_source,
		})
	}

	pub fn axis_discrete(&mut self, client: &mut Client, axis: u32, discrete: i32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:axis_discrete
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 8,
			args: (axis, discrete),
		})
	}
}

impl wl::Object for Pointer {
	fn handle(&mut self, client: &mut Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_pointer:request:set_cursor
				let (_serial, surface, _hotspot_x, _hotspot_y): (
					u32,
					wl::Id<wl::Surface>,
					i32,
					i32,
				) = wlm::decode::from_slice(params)?;

				self.should_hide_cursor = surface.is_null();
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_pointer:request:release
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Pointer"),
		}

		Ok(())
	}
}
