use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct Seat {
	pub object_id: wl::Id<Self>,
	conn: Rc<Connection>,
	keymap: (std::os::fd::RawFd, u64),
}

impl Seat {
	pub fn new(
		object_id: wl::Id<Self>,
		conn: Rc<Connection>,
		keymap: (std::os::fd::RawFd, u64),
	) -> Self {
		Self {
			object_id,
			conn,
			keymap,
		}
	}

	fn capabilities(&self, capabilities: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_seat:event:capabilities
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: capabilities,
		})
	}
}

impl wl::Object for Seat {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:get_pointer
				let id: wl::Id<wl::Pointer> = wlm::decode::from_slice(params)?;
				client.new_object(id, wl::Pointer::new(id, self.conn.clone(), self.object_id));
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:get_keyboard
				let id: wl::Id<wl::Keyboard> = wlm::decode::from_slice(params)?;

				let mut keyboard =
					wl::Keyboard::new(id, self.conn.clone(), self.object_id, self.keymap);
				keyboard.keymap()?;
				keyboard.repeat_info(33, 500)?;

				client.new_object(id, keyboard);
			}
			3 => {
				// https://wayland.app/protocols/wayland#wl_seat:request:release
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Seat"),
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

	fn bind(&self, client: &mut Client, object_id: u32, _version: u32) -> Result<()> {
		let seat = client.new_object(
			wl::Id::new(object_id),
			Self::new(wl::Id::new(object_id), self.conn.clone(), self.keymap),
		);

		seat.capabilities(3)
	}
}
