use crate::{wl, Result};

#[derive(Debug)]
pub struct XdgWmBase {}

impl XdgWmBase {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for XdgWmBase {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_wm_base:request:destroy
			}
			2 => {
				let (id, surface): (u32, u32) = wlm::decode::from_slice(&params)?;
				client.push_client_object(id, wl::XdgSurface::new(id, surface));
			}
			_ => Err(format!("unknown op '{op}' in XdgWmBase"))?,
		}

		Ok(())
	}
}

impl wl::Global for XdgWmBase {
	fn get_name(&self) -> &'static str {
		"xdg_wm_base"
	}

	fn get_version(&self) -> u32 {
		6
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		client.push_client_object(object_id, Self::new());
		Ok(())
	}
}
