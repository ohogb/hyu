use std::io::Write;

use crate::{wl, Result};

#[derive(Debug)]
pub struct Display {
	globals: Vec<Box<dyn wl::Global>>,
}

impl Display {
	pub fn new() -> Self {
		Self {
			globals: Vec::new(),
		}
	}

	pub fn get_global(&self, key: u32) -> Option<&Box<dyn wl::Global>> {
		self.globals.get(key as usize - 1)
	}

	pub fn push_global(&mut self, global: impl wl::Global + 'static) {
		self.globals.push(Box::new(global));
	}
}

impl wl::Object for Display {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		match op {
			0 => {
				let callback: u32 = wlm::decode::from_slice(&params)?;

				let mut buf = Vec::new();

				buf.write_all(&callback.to_ne_bytes())?;
				buf.write_all(&0u16.to_ne_bytes())?;
				buf.write_all(&(8u16 + 4u16).to_ne_bytes())?;
				buf.write_all(&(0u32).to_ne_bytes())?;

				client.get_state().buffer.0.extend(buf);
			}
			1 => {
				let registry_index: u32 = wlm::decode::from_slice(&params)?;
				let registry = wl::Registry::new(registry_index, &self);

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
