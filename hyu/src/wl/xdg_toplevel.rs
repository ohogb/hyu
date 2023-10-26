use crate::{wl, Result};

pub struct XdgToplevel {
	app_id: String,
}

impl XdgToplevel {
	pub fn new() -> Self {
		Self {
			app_id: String::new(),
		}
	}
}

impl wl::Object for XdgToplevel {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			3 => {
				let app_id: String = wlm::decode::from_slice(&params)?;
				self.app_id = app_id;
			}
			_ => Err(format!("unknown op '{op}' in XdgToplevel"))?,
		}

		Ok(())
	}
}
