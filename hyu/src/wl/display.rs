use std::rc::Rc;

use crate::{Client, Connection, Result, state::HwState, wl};

pub struct Display {
	object_id: wl::Id<Self>,
	conn: Rc<Connection>,
	globals: Vec<Box<dyn wl::Global>>,
	started: std::time::Instant,
	serial: u32,
}

impl Display {
	pub fn new(object_id: wl::Id<Self>, conn: Rc<Connection>) -> Self {
		Self {
			object_id,
			conn,
			globals: Vec::new(),
			started: std::time::Instant::now(),
			serial: 0,
		}
	}

	pub fn delete_id<T>(&self, id: wl::Id<T>) -> Result<()> {
		// https://wayland.app/protocols/wayland#wl_display:event:delete_id
		self.conn.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: id,
		})
	}

	pub fn get_global(&self, key: u32) -> Option<&(dyn wl::Global)> {
		self.globals.get(key as usize - 1).map(|x| &**x)
	}

	pub fn push_global(&mut self, global: impl wl::Global + 'static) {
		self.globals.push(Box::new(global));
	}

	pub fn get_time(&self) -> std::time::Duration {
		self.started.elapsed()
	}

	pub fn new_serial(&mut self) -> u32 {
		let ret = self.serial;
		self.serial += 1;

		ret
	}
}

impl wl::Object for Display {
	fn handle(
		&mut self,
		client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_display:request:sync
				let callback: wl::Id<wl::Callback> = wlm::decode::from_slice(params)?;

				let callback = client
					.new_object(callback, wl::Callback::new(callback, self.conn.clone()))
					.clone();

				callback.done(client, self.serial)?;
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_display:request:get_registry
				let registry_index: wl::Id<wl::Registry> = wlm::decode::from_slice(params)?;

				let registry = client.new_object(
					registry_index,
					wl::Registry::new(registry_index, self.conn.clone(), self.object_id),
				);

				for (index, global) in self.globals.iter().enumerate() {
					registry.global(index as u32 + 1, global.get_name(), global.get_version())?;
				}
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Display"),
		}

		Ok(())
	}
}
