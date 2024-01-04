use crate::{wl, Result};

pub struct XdgSurface {
	object_id: u32,
}

impl XdgSurface {
	pub fn new(object_id: u32) -> Self {
		Self { object_id }
	}

	pub fn configure(&self, client: &mut wl::Client) -> Result<()> {
		let mut buf = Vec::new();

		buf.extend(self.object_id.to_ne_bytes());
		buf.extend(0u16.to_ne_bytes());

		let arg = wlm::encode::to_vec(&123).unwrap();

		buf.extend((8u16 + arg.len() as u16).to_ne_bytes());
		buf.extend(arg);

		client.get_state().buffer.0.extend(buf);

		Ok(())
	}
}

impl wl::Object for XdgSurface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			1 => {
				let id: u32 = wlm::decode::from_slice(&params)?;

				let xdg_toplevel = wl::XdgToplevel::new(id, self);
				xdg_toplevel.configure(client)?;

				client.push_client_object(id, xdg_toplevel);
			}
			_ => Err(format!("unknown op '{op}' in XdgSurface"))?,
		}

		Ok(())
	}
}
