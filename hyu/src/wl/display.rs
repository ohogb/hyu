use crate::{wl, Result};

pub struct Display {
	object_id: wl::Id<Self>,
	globals: Vec<Box<dyn wl::Global + Send + Sync>>,
}

impl Display {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self {
			object_id,
			globals: Vec::new(),
		}
	}

	// https://wayland.app/protocols/wayland#wl_display:event:delete_id
	pub fn delete_id<T>(&self, client: &mut wl::Client, id: wl::Id<T>) -> Result<()> {
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: id,
		})
	}

	pub fn get_global(&self, key: u32) -> Option<&(dyn wl::Global + Send + Sync)> {
		self.globals.get(key as usize - 1).map(|x| &**x)
	}

	pub fn push_global(&mut self, global: impl wl::Global + Send + Sync + 'static) {
		self.globals.push(Box::new(global));
	}
}

impl wl::Object for Display {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_display:request:sync
				let callback: wl::Id<wl::Callback> = wlm::decode::from_slice(params)?;

				let callback = client
					.new_object(callback, wl::Callback::new(callback))
					.clone();

				callback.done(client, 0)?;
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_display:request:get_registry
				let registry_index: wl::Id<wl::Registry> = wlm::decode::from_slice(params)?;
				let registry = wl::Registry::new(registry_index, self.object_id);

				for (index, global) in self.globals.iter().enumerate() {
					registry.global(
						client,
						index as u32 + 1,
						global.get_name(),
						global.get_version(),
					)?;
				}

				client.new_object(registry_index, registry);
			}
			_ => Err(format!("unknown op '{op}' in Display"))?,
		}

		Ok(())
	}
}
