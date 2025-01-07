use crate::{Client, Result, state::HwState, wl};

pub struct DataSource {
	object_id: wl::Id<Self>,
}

impl DataSource {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}
}

impl wl::Object for DataSource {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_data_source:request:offer
				let _mime_type: String = wlm::decode::from_slice(params)?;
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_data_source:request:destroy
				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_data_source:request:set_actions
				let _dnd_actions: u32 = wlm::decode::from_slice(params)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in DataSource"),
		}

		Ok(())
	}
}
