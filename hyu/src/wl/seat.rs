use crate::{wl, Point, Result};

pub struct Seat {
	pub object_id: wl::Id<Self>,
	serial: u32,
	pub pointer_position: Point,
	pub moving_toplevel: Option<(wl::Id<wl::XdgToplevel>, Point, Point)>,
	keymap: (std::os::fd::RawFd, u64),
}

impl Seat {
	pub fn new(object_id: wl::Id<Self>, keymap: (std::os::fd::RawFd, u64)) -> Self {
		Self {
			object_id,
			serial: 0,
			pointer_position: Point(0, 0),
			moving_toplevel: None,
			keymap,
		}
	}

	fn capabilities(&self, client: &mut wl::Client, capabilities: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_seat:event:capabilities
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: capabilities,
		})
	}

	pub fn serial(&mut self) -> u32 {
		let ret = self.serial;
		self.serial += 1;

		ret
	}

	pub fn start_moving_toplevel(&mut self, toplevel: &wl::XdgToplevel) {
		self.moving_toplevel = Some((toplevel.object_id, toplevel.position, self.pointer_position));
	}
}

impl wl::Object for Seat {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:get_pointer
				let id: wl::Id<wl::Pointer> = wlm::decode::from_slice(params)?;
				client.new_object(id, wl::Pointer::new(id, self.object_id));
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:get_keyboard
				let id: wl::Id<wl::Keyboard> = wlm::decode::from_slice(params)?;

				let mut keyboard = wl::Keyboard::new(id, self.object_id, self.keymap);
				keyboard.keymap(client)?;
				keyboard.repeat_info(client, 500, 500)?;

				client.new_object(id, keyboard);
			}
			3 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:release
			}
			_ => Err(format!("unknown op '{op}' in Seat"))?,
		}

		Ok(())
	}
}

impl wl::Global for Seat {
	fn get_name(&self) -> &'static str {
		"wl_seat"
	}

	fn get_version(&self) -> u32 {
		7
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		let seat = client.new_object(
			wl::Id::new(object_id),
			Self::new(wl::Id::new(object_id), self.keymap),
		);

		seat.capabilities(client, 3)
	}
}
