use crate::{wl, Result};

pub struct DataSource {
	object_id: wl::Id<Self>,
}

impl DataSource {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}
}

impl wl::Object for DataSource {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_data_source:request:offer
				let _mime_type: String = wlm::decode::from_slice(params)?;
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_data_source:request:destroy
				client.remove_object(self.object_id)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in DataSource"),
		}

		Ok(())
	}
}
