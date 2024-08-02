use std::{io::Read as _, os::fd::AsRawFd as _};

use crate::{rt::Producer, Result};

pub struct Drm {
	fd: std::os::fd::RawFd,
}

impl Drm {
	pub fn new(fd: std::os::fd::RawFd) -> Self {
		Self { fd }
	}
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

pub enum DrmMessage {
	PageFlip {
		tv_sec: u32,
		tv_usec: u32,
		sequence: u32,
		crtc_id: u32,
	},
}

impl Producer for Drm {
	type Message<'a> = DrmMessage;
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

		callback(DrmMessage::PageFlip {
			tv_sec: drm_event_vblank.tv_sec,
			tv_usec: drm_event_vblank.tv_usec,
			sequence: drm_event_vblank.sequence,
			crtc_id: drm_event_vblank.crtc_id,
		})?;

		Ok(std::ops::ControlFlow::Continue(()))
	}
}
