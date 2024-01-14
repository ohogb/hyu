use crate::{wl, Result};

#[derive(Debug)]
pub struct Display {
	object_id: u32,
	globals: Vec<Box<dyn wl::Global + Send + Sync>>,
}

impl Display {
	pub fn new(object_id: u32) -> Self {
		Self {
			object_id,
			globals: Vec::new(),
		}
	}

	pub fn get_global(&self, key: u32) -> Option<&(dyn wl::Global + Send + Sync)> {
		self.globals.get(key as usize - 1).map(|x| &**x)
	}

	pub fn push_global(&mut self, global: impl wl::Global + Send + Sync + 'static) {
		self.globals.push(Box::new(global));
	}
}

impl wl::Object for Display {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				let callback: u32 = wlm::decode::from_slice(&params)?;

				client.send_message(wlm::Message {
					object_id: callback,
					op: 0,
					args: 0u32,
				})?;
			}
			1 => {
				let registry_index: u32 = wlm::decode::from_slice(&params)?;
				let registry = wl::Registry::new(registry_index, self.object_id);

				for (index, global) in self.globals.iter().enumerate() {
					let message = registry.global(
						index as u32 + 1,
						global.get_name(),
						global.get_version(),
					)?;

					client.get_state().buffer.0.extend(message);
				}

				client.push_client_object(registry_index, registry);
			}
			_ => Err(format!("unknown op '{op}' in Display"))?,
		}

		Ok(())
	}
}
