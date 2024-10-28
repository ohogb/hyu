use crate::{Client, Result, wl};

pub struct ZwlrLayerSurfaceV1 {
	object_id: wl::Id<Self>,
}

impl ZwlrLayerSurfaceV1 {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}

	pub fn configure(
		&self,
		client: &mut wl::Client,
		serial: u32,
		width: u32,
		height: u32,
	) -> Result<()> {
		// https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:event:configure
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: (serial, width, height),
		})
	}
}

impl wl::Object for ZwlrLayerSurfaceV1 {
	fn handle(&mut self, client: &mut Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_size
				let (_width, _height): (u32, u32) = wlm::decode::from_slice(params)?;
			}
			1 => {
				// https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_anchor
				let _anchor: u32 = wlm::decode::from_slice(params)?;
			}
			2 => {
				// https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_exclusive_zone
				let _zone: i32 = wlm::decode::from_slice(params)?;
			}
			3 => {
				// https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_margin
				let (_top, _right, _bottom, _left): (i32, i32, i32, i32) =
					wlm::decode::from_slice(params)?;
			}
			4 => {
				// https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_keyboard_interactivity
				let _keyboard_interactivity: u32 = wlm::decode::from_slice(params)?;
			}
			6 => {
				// https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:ack_configure
				let _serial: u32 = wlm::decode::from_slice(params)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in ZwlrLayerSurfaceV1"),
		}

		Ok(())
	}
}
