use crate::{Client, Result, wl};

#[derive(Clone)]
pub struct Callback {
	object_id: wl::Id<Self>,
}

impl Callback {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}

	pub fn done(self, client: &mut Client, data: u32) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_callback:event:done
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: data,
		})?;

		unsafe {
			client.remove_object(self.object_id)?;
		}

		Ok(())
	}
}

impl wl::Object for Callback {
	fn handle(&mut self, _client: &mut Client, op: u16, _params: &[u8]) -> Result<()> {
		color_eyre::eyre::bail!("unknown op '{op}' in Callback");
	}
}
