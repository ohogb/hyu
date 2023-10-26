use crate::{wl, Result};

pub struct XdgSurface {}

impl XdgSurface {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for XdgSurface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			1 => {
				let id: u32 = wlm::decode::from_slice(&params)?;
				client.push_client_object(id, wl::XdgToplevel::new());
			}
			_ => Err(format!("unknown op '{op}' in XdgSurface"))?,
		}

		Ok(())
	}
}
