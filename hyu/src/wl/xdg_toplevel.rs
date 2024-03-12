use crate::{wl, Result};

pub struct XdgToplevel {
	object_id: u32,
	pub surface: u32,
	app_id: String,
	title: String,
	pub position: (i32, i32),
}

impl XdgToplevel {
	pub fn new(
		client: &mut wl::Client,
		object_id: u32,
		surface: u32,
		position: (i32, i32),
	) -> Self {
		client.add_window(object_id);

		Self {
			object_id,
			surface,
			app_id: String::new(),
			title: String::new(),
			position,
		}
	}

	pub fn configure(&self, client: &mut wl::Client) -> Result<()> {
		client.send_message(wlm::Message {
			object_id: self.object_id,
			op: 0,
			args: (0u32, 0u32, [0u32]),
		})?;

		let xdg_surface = client.get_object::<wl::XdgSurface>(self.surface)?;
		xdg_surface.configure(client)?;

		Ok(())
	}
}

impl wl::Object for XdgToplevel {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:destroy
			}
			1 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_parent
				let _parent: u32 = wlm::decode::from_slice(&params)?;
			}
			2 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_title
				let title: String = wlm::decode::from_slice(&params)?;
				self.title = title;
			}
			3 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_app_id
				let app_id: String = wlm::decode::from_slice(&params)?;
				self.app_id = app_id;
			}
			7 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_max_size
				let (_width, _height): (i32, i32) = wlm::decode::from_slice(&params)?;
			}
			8 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_min_size
				let (_width, _height): (u32, u32) = wlm::decode::from_slice(&params)?;
			}
			10 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:unset_maximized
			}
			12 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:unset_fullscreen
			}
			_ => Err(format!("unknown op '{op}' in XdgToplevel"))?,
		}

		Ok(())
	}
}
