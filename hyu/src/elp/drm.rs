use std::io::Read as _;

use crate::{elp, Result};

pub struct Source {
	fd: std::os::fd::RawFd,
}

#[repr(C)]
struct DrmEventVBlank {
	typee: u32,
	length: u32,
	user_data: u64,
	tv_sec: u32,
	tv_usec: u32,
	sequence: u32,
	crtc_id: u32,
}

pub enum Message {
	PageFlip {
		tv_sec: u32,
		tv_usec: u32,
		sequence: u32,
		crtc_id: u32,
	},
}

impl elp::Source for Source {
	type Message<'a> = Message;
	type Ret = Result<()>;

	fn fd(&self) -> std::os::fd::RawFd {
		self.fd
	}

	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>> {
		let mut ret = [0u8; 0x1000];

		let amount = nix::unistd::read(self.fd, &mut ret)?;
		assert!(amount == std::mem::size_of::<DrmEventVBlank>());

		let mut ret = &ret[..amount];

		let mut drm_event_vblank = [0u8; std::mem::size_of::<DrmEventVBlank>()];
		ret.read_exact(&mut drm_event_vblank)?;

		let drm_event_vblank =
			unsafe { &mut *(&mut drm_event_vblank as *mut _ as *mut DrmEventVBlank) };

		assert!(drm_event_vblank.typee == 2);

		callback(Message::PageFlip {
			tv_sec: drm_event_vblank.tv_sec,
			tv_usec: drm_event_vblank.tv_usec,
			sequence: drm_event_vblank.sequence,
			crtc_id: drm_event_vblank.crtc_id,
		})?;

		Ok(std::ops::ControlFlow::Continue(()))
	}
}

pub fn create(fd: std::os::fd::RawFd) -> Source {
	Source { fd }
}
