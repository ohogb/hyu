use crate::{wl, Result};

pub struct DataDevice {
	object_id: wl::Id<Self>,
	_seat: wl::Id<wl::Seat>,
}

impl DataDevice {
	pub fn new(object_id: wl::Id<Self>, seat: wl::Id<wl::Seat>) -> Self {
		Self {
			object_id,
			_seat: seat,
		}
	}
}

impl wl::Object for DataDevice {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_data_device:request:start_drag
				let (_source, _origin, _icon, _serial): (
					wl::Id<wl::DataSource>,
					wl::Id<wl::Surface>,
					wl::Id<wl::Surface>,
					u32,
				) = wlm::decode::from_slice(params)?;
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_data_device:request:set_selection
				let (_source, _serial): (wl::Id<wl::DataSource>, u32) =
					wlm::decode::from_slice(params)?;
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_data_device:request:release
				client.remove_object(self.object_id)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in DataDevice"),
		}

		Ok(())
	}
}
