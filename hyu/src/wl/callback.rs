use crate::{wl, Result};

pub struct Callback {
	object_id: wl::Id<Self>,
}

impl Callback {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}

	pub fn done(self, client: &mut wl::Client, data: u32) -> Result<()> {
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: data,
		})?;

		client.remove_object(self.object_id)
	}
}

impl wl::Object for Callback {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: &[u8]) -> Result<()> {
		Err(format!("unknown op '{op}' in Callback"))?
	}
}
