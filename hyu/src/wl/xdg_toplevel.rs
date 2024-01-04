use crate::{wl, Result};

pub struct XdgToplevel {
	object_id: u32,
	surface: *const wl::XdgSurface,
	app_id: String,
	title: String,
}

impl XdgToplevel {
	pub fn new(object_id: u32, surface: &wl::XdgSurface) -> Self {
		Self {
			object_id,
			surface: surface as _,
			app_id: String::new(),
			title: String::new(),
		}
	}

	pub fn configure(&self, client: &mut wl::Client) -> Result<()> {
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 0,
			args: (0u32, 0u32, [0u32]),
		})?;

		unsafe {
			(*self.surface).configure(client)?;
		}

		Ok(())
	}
}

impl wl::Object for XdgToplevel {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			2 => {
				let title: String = wlm::decode::from_slice(&params)?;
				self.title = title;
			}
			3 => {
				let app_id: String = wlm::decode::from_slice(&params)?;
				self.app_id = app_id;
			}
			_ => Err(format!("unknown op '{op}' in XdgToplevel"))?,
		}

		Ok(())
	}
}
