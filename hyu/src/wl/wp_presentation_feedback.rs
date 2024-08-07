use nix::sys::time::TimeValLike;

use crate::{wl, Result};

pub struct WpPresentationFeedback {
	object_id: wl::Id<Self>,
}

impl WpPresentationFeedback {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}

	pub fn sync_output(&self, client: &mut wl::Client, output: wl::Id<wl::Output>) -> Result<()> {
		// https://wayland.app/protocols/presentation-time#wp_presentation_feedback:event:sync_output
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: output,
		})
	}

	pub fn presented(
		&self,
		client: &mut wl::Client,
		time: std::time::Duration,
		till_next_refresh: u32,
		sequence: u64,
		flags: u32,
	) -> Result<()> {
		// https://wayland.app/protocols/presentation-time#wp_presentation_feedback:event:presented
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: (
				((time.as_secs() as u64 >> 32) & 0xFFFFFFFF) as u32,
				((time.as_secs() as u64) & 0xFFFFFFFF) as u32,
				(((time.as_nanos() % 1_000_000_000) as u64) & 0xFFFFFFFF) as u32,
				till_next_refresh,
				((sequence >> 32) & 0xFFFFFFFF) as u32,
				(sequence & 0xFFFFFFFF) as u32,
				flags,
			),
		})?;

		client.remove_object(self.object_id)
	}
}

impl wl::Object for WpPresentationFeedback {
	fn handle(&mut self, _client: &mut wl::Client, op: u16, _params: &[u8]) -> Result<()> {
		match op {
			_ => Err(format!("unknown op '{op}' in WpPresentationFeedback"))?,
		}

		Ok(())
	}
}
