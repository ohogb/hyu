use std::rc::Rc;

use crate::{
	Client, Connection, Point, Result,
	state::{self, HwState},
	wl,
};

pub struct XdgToplevel {
	pub object_id: wl::Id<Self>,
	conn: Rc<Connection>,
	pub surface: wl::Id<wl::XdgSurface>,
	app_id: String,
	title: String,
	pub position: Point,
	pub size: Option<Point>,
	pub states: Vec<u32>,
}

impl XdgToplevel {
	pub fn new(
		client: &mut Client,
		object_id: wl::Id<Self>,
		conn: Rc<Connection>,
		surface: wl::Id<wl::XdgSurface>,
		position: Point,
		fd: std::os::fd::RawFd,
	) -> Self {
		client.changes.push(state::Change::Push(fd, object_id));

		Self {
			object_id,
			conn,
			surface,
			app_id: String::new(),
			title: String::new(),
			position,
			size: None,
			states: Vec::new(),
		}
	}

	pub fn configure(&self, client: &mut Client) -> Result<()> {
		let size = self.size.unwrap_or(Point(0, 0));

		// https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:configure
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (size.0, size.1, &self.states),
		})?;

		let xdg_surface = client.get_object_mut(self.surface)?;
		xdg_surface.configure()
	}

	pub fn close(&self) -> Result<()> {
		// https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:close
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: (),
		})
	}

	pub fn configure_bounds(&self, width: i32, height: i32) -> Result<()> {
		// https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:configure_bounds
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 2,
			args: (width, height),
		})
	}

	pub fn add_state(&mut self, state: u32) {
		if !self.states.contains(&state) {
			self.states.push(state);
		}
	}

	pub fn remove_state(&mut self, state: u32) {
		self.states.retain(|&x| x != state);
	}
}

impl wl::Object for XdgToplevel {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:destroy
				client
					.changes
					.push(state::Change::RemoveToplevel(client.fd, self.object_id));

				unsafe {
					client.remove_object(self.object_id)?;
				}
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
				let (_seat, _serial): (wl::Id<wl::Seat>, u32) = wlm::decode::from_slice(params)?;
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
				let _output: wl::Id<wl::Output> = wlm::decode::from_slice(params)?;

				self.add_state(2);
				self.configure(client)?;
			}
			12 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:unset_fullscreen
				self.remove_state(2);
				self.configure(client)?;
			}
			13 => {
				// https://wayland.app/protocols/xdg-shell#xdg_toplevel:request:set_minimized
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in XdgToplevel"),
		}

		Ok(())
	}
}
