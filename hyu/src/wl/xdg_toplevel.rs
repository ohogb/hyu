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
		let mut buf = Vec::new();

		buf.extend(self.object_id.to_ne_bytes());
		buf.extend(0u16.to_ne_bytes());

		let arg = wlm::encode::to_vec(&(0, 0, [0])).unwrap();

		buf.extend((8u16 + arg.len() as u16).to_ne_bytes());
		buf.extend(arg);

		client.get_state().buffer.0.extend(buf);

		unsafe {
			(*self.surface).configure(client)?;
		}

		Ok(())
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
