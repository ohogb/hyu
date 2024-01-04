use crate::{wl, Result};

#[derive(Debug)]
pub struct Output {}

impl Output {
	pub fn new() -> Self {
		Self {}
	}
}

impl wl::Object for Output {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: Vec<u8>) -> Result<()> {
		todo!()
	}
}

impl wl::Global for Output {
	fn get_name(&self) -> &'static str {
		"wl_output"
	}

	fn get_version(&self) -> u32 {
		3
	}

	fn bind(&self, client: &mut wl::Client, object_id: u32) -> Result<()> {
		client.push_client_object(object_id, Self::new());

		let mut buf = Vec::new();

		buf.extend(object_id.to_ne_bytes());
		buf.extend(0u16.to_ne_bytes());

		let arg =
			wlm::encode::to_vec(&(0u32, 0u32, 600u32, 340u32, 0u32, "AUS", "ROG XG27AQM", 0u32))
				.unwrap();

		buf.extend((8u16 + arg.len() as u16).to_ne_bytes());
		buf.extend(arg);

		client.get_state().buffer.0.extend(buf);

		let mut buf = Vec::new();

		buf.extend(object_id.to_ne_bytes());
		buf.extend(1u16.to_ne_bytes());

		let arg = wlm::encode::to_vec(&(3u32, 2560u32, 1440u32, 270000u32)).unwrap();

		buf.extend((8u16 + arg.len() as u16).to_ne_bytes());
		buf.extend(arg);

		client.get_state().buffer.0.extend(buf);

		let mut buf = Vec::new();

		buf.extend(object_id.to_ne_bytes());
		buf.extend(3u16.to_ne_bytes());

		let arg = wlm::encode::to_vec(&(1u32)).unwrap();

		buf.extend((8u16 + arg.len() as u16).to_ne_bytes());
		buf.extend(arg);

		client.get_state().buffer.0.extend(buf);

		let mut buf = Vec::new();

		buf.extend(object_id.to_ne_bytes());
		buf.extend(2u16.to_ne_bytes());

		let arg = wlm::encode::to_vec(&()).unwrap();

		buf.extend((8u16 + arg.len() as u16).to_ne_bytes());
		buf.extend(arg);

		client.get_state().buffer.0.extend(buf);

		Ok(())
	}
}
