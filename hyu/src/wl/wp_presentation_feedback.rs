use crate::{Client, Result, state::HwState, wl};

pub struct WpPresentationFeedback {
	object_id: wl::Id<Self>,
}

impl WpPresentationFeedback {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self { object_id }
	}

	pub fn sync_output(&self, client: &mut Client, output: wl::Id<wl::Output>) -> Result<()> {
		// https://wayland.app/protocols/presentation-time#wp_presentation_feedback:event:sync_output
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 0,
			args: output,
		})
	}

	pub fn presented(
		&self,
		client: &mut Client,
		time: std::time::Duration,
		till_next_refresh: std::time::Duration,
		sequence: u64,
		flags: u32,
	) -> Result<()> {
		// https://wayland.app/protocols/presentation-time#wp_presentation_feedback:event:presented
		client.send_message(wlm::Message {
			object_id: *self.object_id,
			op: 1,
			args: (
				((time.as_secs() >> 32) & 0xFFFFFFFF) as u32,
				(time.as_secs() & 0xFFFFFFFF) as u32,
				(((time.as_nanos() % 1_000_000_000) as u64) & 0xFFFFFFFF) as u32,
				till_next_refresh.as_nanos() as u32,
				((sequence >> 32) & 0xFFFFFFFF) as u32,
				(sequence & 0xFFFFFFFF) as u32,
				flags,
			),
		})?;

		unsafe {
			client.remove_object(self.object_id)?;
		}

		Ok(())
	}
}

impl wl::Object for WpPresentationFeedback {
	fn handle(
		&mut self,
		_client: &mut Client,
		_hw_state: &mut HwState,
		op: u16,
		_params: &[u8],
	) -> Result<()> {
		color_eyre::eyre::bail!("unknown op '{op}' in WpPresentationFeedback");
	}
}
