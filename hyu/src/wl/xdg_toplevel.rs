use crate::{state, wl, Result};

pub struct XdgToplevel {
	pub object_id: wl::Id<Self>,
	pub surface: wl::Id<wl::XdgSurface>,
	app_id: String,
	title: String,
	pub position: (i32, i32),
	pub size: Option<(i32, i32)>,
}

impl XdgToplevel {
	pub fn new(
		object_id: wl::Id<Self>,
		surface: wl::Id<wl::XdgSurface>,
		position: (i32, i32),
		fd: std::os::fd::RawFd,
	) -> Self {
		state::CHANGES
			.lock()
			.unwrap()
			.push(state::Change::Push(fd, object_id));

		Self {
			object_id,
			surface,
			app_id: String::new(),
			title: String::new(),
			position,
			size: None,
		}
	}

	pub fn configure(
		&self,
		client: &mut wl::Client,
		width: i32,
		height: i32,
		states: &[u32],
	) -> Result<()> {
		// https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:configure
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (width, height, states),
		})?;

		let xdg_surface = client.get_object_mut(self.surface)?;
		xdg_surface.configure(client)
	}

	pub fn configure_bounds(&self, client: &mut wl::Client, width: i32, height: i32) -> Result<()> {
		// https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:configure_bounds
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 2,
			args: (width, height),
		})
	}
}

impl wl::Object for XdgToplevel {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:destroy
				state::CHANGES
					.lock()
					.unwrap()
					.push(state::Change::RemoveToplevel(client.fd, self.object_id));

				client.remove_object(self.object_id)?;
			}
			1 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_parent
				let _parent: u32 = wlm::decode::from_slice(params)?;
			}
			2 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_title
				let title: String = wlm::decode::from_slice(params)?;
				self.title = title;
			}
			3 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_app_id
				let app_id: String = wlm::decode::from_slice(params)?;
				self.app_id = app_id;
			}
			5 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:move
				let (seat, _serial): (wl::Id<wl::Seat>, u32) = wlm::decode::from_slice(params)?;

				let seat = client.get_object_mut(seat)?;
				seat.start_moving_toplevel(self);
			}
			6 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:resize
				let (_seat, _serial, _edges): (wl::Id<wl::Seat>, u32, u32) =
					wlm::decode::from_slice(params)?;
			}
			7 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_max_size
				let (_width, _height): (i32, i32) = wlm::decode::from_slice(params)?;
			}
			8 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_min_size
				let (_width, _height): (u32, u32) = wlm::decode::from_slice(params)?;
			}
			9 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_maximized
			}
			10 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:unset_maximized
			}
			11 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_fullscreen
			}
			12 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:unset_fullscreen
			}
			13 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_minimized
			}
			_ => Err(format!("unknown op '{op}' in XdgToplevel"))?,
		}

		Ok(())
	}
}
