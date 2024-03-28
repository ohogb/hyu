use crate::{wl, Result};

pub struct Pointer {
	object_id: wl::Id<Self>,
	seat_id: wl::Id<wl::Seat>,
}

impl Pointer {
	pub fn new(object_id: wl::Id<Self>, seat_id: wl::Id<wl::Seat>) -> Self {
		Self { object_id, seat_id }
	}

	pub fn enter(
		&mut self,
		client: &mut wl::Client,
		surface: wl::Id<wl::Surface>,
		x: i32,
		y: i32,
	) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:enter
		let seat = client.get_object_mut(self.seat_id)?;

		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (
				seat.serial(),
				surface,
				fixed::types::I24F8::from_num(x),
				fixed::types::I24F8::from_num(y),
			),
		})?;

		Ok(())
	}

	pub fn leave(&mut self, client: &mut wl::Client, surface: wl::Id<wl::Surface>) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:leave
		let seat = client.get_object_mut(self.seat_id)?;

		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: (seat.serial(), surface),
		})?;

		Ok(())
	}

	pub fn motion(&mut self, client: &mut wl::Client, x: i32, y: i32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:motion
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 2,
			args: (
				0,
				fixed::types::I24F8::from_num(x),
				fixed::types::I24F8::from_num(y),
			),
		})?;

		Ok(())
	}

	pub fn button(&mut self, client: &mut wl::Client, button: u32, state: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:button
		let seat = client.get_object_mut(self.seat_id)?;

		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 3,
			args: (seat.serial(), 0, button, state),
		})?;

		Ok(())
	}

	pub fn frame(&mut self, client: &mut wl::Client) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_pointer:event:frame
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 5,
			args: (),
		})?;

		Ok(())
	}
}

impl wl::Object for Pointer {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_pointer:request:set_cursor
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_pointer:request:release
			}
			_ => Err(format!("unknown op '{op}' in Pointer"))?,
		}

		Ok(())
	}
}
