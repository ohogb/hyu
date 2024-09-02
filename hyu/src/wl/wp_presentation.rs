use crate::{wl, Result};

pub struct WpPresentation {
	object_id: wl::Id<Self>,
}

impl WpPresentation {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}

	pub fn clock_id(&self, client: &mut wl::Client, clock_id: u32) -> Result<()> {
		// https://wayland.app/protocols/presentation-time#wp_presentation:event:clock_id
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: clock_id,
		})
	}
}

impl wl::Object for WpPresentation {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/presentation-time#wp_presentation:request:destroy
				client.remove_object(self.object_id)?;
			}
			1 => {
				// https://wayland.app/protocols/presentation-time#wp_presentation:request:feedback
				let (surface, callback): (wl::Id<wl::Surface>, wl::Id<wl::WpPresentationFeedback>) =
					wlm::decode::from_slice(params)?;

				client.new_object(callback, wl::WpPresentationFeedback::new(callback));

				let surface = client.get_object_mut(surface)?;
				surface.pending_presentation_feedback = Some(callback);
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

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		let object_id = wl::Id::new(object_id);
		let object = client.new_object(object_id, Self::new(object_id));

		object.clock_id(client, 1)?;

		Ok(())
	}
}
