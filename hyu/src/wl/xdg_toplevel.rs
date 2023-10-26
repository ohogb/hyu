use crate::{wl, Result};

pub struct XdgToplevel {
	app_id: String,
	title: String,
}

impl XdgToplevel {
	pub fn new() -> Self {
		Self {
			app_id: String::new(),
			title: String::new(),
		}
	}
}

impl wl::Object for XdgToplevel {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
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
