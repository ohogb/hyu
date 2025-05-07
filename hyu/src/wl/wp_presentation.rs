use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct WpPresentation {
	object_id: wl::Id<Self>,
	conn: Rc<Connection>,
}

impl WpPresentation {
	pub fn new(object_id: wl::Id<Self>, conn: Rc<Connection>) -> Self {
		Self { object_id, conn }
	}

	pub fn clock_id(&self, clock_id: u32) -> Result<()> {
		// https://wayland.app/protocols/presentation-time#wp_presentation:event:clock_id
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: clock_id,
		})
	}
}

impl wl::Object for WpPresentation {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/presentation-time#wp_presentation:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			1 => {
				// https://wayland.app/protocols/presentation-time#wp_presentation:request:feedback
				let (surface, callback): (wl::Id<wl::Surface>, wl::Id<wl::WpPresentationFeedback>) =
					wlm::decode::from_slice(params)?;

				client.new_object(
					callback,
					wl::WpPresentationFeedback::new(callback, self.conn.clone()),
				);

				let surface = client.get_object_mut(surface)?;
				surface.pending.presentation_feedback = Some(callback);
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in WpPresentation"),
		}

		Ok(())
	}
}

impl wl::Global for WpPresentation {
	fn get_name(&self) -> &'static str {
		"wp_presentation"
	}

	fn get_version(&self) -> u32 {
		1
	}

	fn bind(&self, client: &mut Client, object_id: u32, _version: u32) -> Result<()> {
		let object_id = wl::Id::new(object_id);
		let object = client.new_object(object_id, Self::new(object_id, self.conn.clone()));

		object.clock_id(1)?;

		Ok(())
	}
}
