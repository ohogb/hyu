use std::os::fd::AsRawFd as _;

use crate::Result;

pub struct Device {
	file: std::fs::File,
}

impl Device {
	pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self> {
		Ok(Self {
			file: std::fs::OpenOptions::new()
				.read(true)
				.write(true)
				.open(path)?,
		})
	}

	pub fn get_resources(&self) -> Result<Card> {
		nix::ioctl_readwrite!(func, 'd', 0xA0, Card);

		let mut ret = std::mem::MaybeUninit::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			func(self.file.as_raw_fd(), ptr)?;

			if (*ptr).count_fbs > 0 {
				(*ptr).fb_id_ptr = Box::leak(vec![0u32; (*ptr).count_fbs as _].into_boxed_slice())
					.as_mut_ptr() as u64;
			}

			if (*ptr).count_crtcs > 0 {
				(*ptr).crtc_id_ptr =
					Box::leak(vec![0u32; (*ptr).count_crtcs as _].into_boxed_slice()).as_mut_ptr()
						as u64;
			}

			if (*ptr).count_connectors > 0 {
				(*ptr).connected_id_ptr =
					Box::leak(vec![0u32; (*ptr).count_connectors as _].into_boxed_slice())
						.as_mut_ptr() as u64;
			}

			if (*ptr).count_encoders > 0 {
				(*ptr).encoder_id_ptr =
					Box::leak(vec![0u32; (*ptr).count_encoders as _].into_boxed_slice())
						.as_mut_ptr() as u64;
			}

			func(self.file.as_raw_fd(), ptr)?;
		};

		Ok(unsafe { ret.assume_init() })
	}

	pub fn get_connector(&self, connector_id: u32) -> Result<Connector> {
		nix::ioctl_readwrite!(func, 'd', 0xA7, Connector);

		let mut ret = std::mem::MaybeUninit::<Connector>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).connector_id = connector_id;

			func(self.file.as_raw_fd(), ptr).unwrap();

			if (*ptr).count_props > 0 {
				(*ptr).props_ptr = Box::leak(vec![0u32; (*ptr).count_props as _].into_boxed_slice())
					.as_mut_ptr() as u64;

				(*ptr).prop_values_ptr =
					Box::leak(vec![0u64; (*ptr).count_props as _].into_boxed_slice()).as_mut_ptr()
						as u64;
			}

			if (*ptr).count_modes > 0 {
				(*ptr).modes_ptr = Box::leak(
					vec![[0u8; std::mem::size_of::<ModeInfo>()]; (*ptr).count_modes as _]
						.into_boxed_slice(),
				)
				.as_mut_ptr() as u64;
			}

			if (*ptr).count_encoders > 0 {
				(*ptr).encoders_ptr =
					Box::leak(vec![0u32; (*ptr).count_encoders as _].into_boxed_slice())
						.as_mut_ptr() as u64;
			}

			func(self.file.as_raw_fd(), ptr).unwrap();
		};

		Ok(unsafe { ret.assume_init() })
	}

	pub fn get_encoder(&self, encoder_id: u32) -> Result<Encoder> {
		nix::ioctl_readwrite!(func, 'd', 0xA6, Encoder);

		let mut ret = std::mem::MaybeUninit::<Encoder>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).encoder_id = encoder_id;

			func(self.file.as_raw_fd(), ptr).unwrap();
		}

		Ok(unsafe { ret.assume_init() })
	}

	pub fn get_fd(&self) -> std::os::fd::RawFd {
		self.file.as_raw_fd()
	}

	pub fn add_fb(
		&self,
		width: u32,
		height: u32,
		depth: u8,
		bpp: u8,
		pitch: u32,
		bo: u32,
	) -> Result<u32> {
		nix::ioctl_readwrite!(func, 'd', 0xAE, FbCmd);

		let mut ret = std::mem::MaybeUninit::<FbCmd>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).width = width;
			(*ptr).height = height;
			(*ptr).depth = depth as _;
			(*ptr).bpp = bpp as _;
			(*ptr).pitch = pitch;
			(*ptr).handle = bo;

			func(self.file.as_raw_fd(), ptr)?;
		}

		Ok(unsafe { ret.assume_init() }.fb_id)
	}

	pub fn set_crtc(
		&self,
		crtc_id: u32,
		fb_id: u32,
		x: u32,
		y: u32,
		connectors: &[u32],
		mode: &ModeInfo,
	) -> Result<()> {
		nix::ioctl_readwrite!(func, 'd', 0xA2, Crtc);

		let mut ret = std::mem::MaybeUninit::<Crtc>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).crtc_id = crtc_id;
			(*ptr).fb_id = fb_id;
			(*ptr).x = x;
			(*ptr).y = y;
			(*ptr).set_connectors_ptr = connectors.as_ptr() as _;
			(*ptr).count_connectors = connectors.len() as _;

			(*ptr).mode = mode.clone();
			(*ptr).mode_valid = 1;

			func(self.file.as_raw_fd(), ptr)?;
		}

		Ok(())
	}

	pub fn page_flip(
		&self,
		crtc_id: u32,
		fb_id: u32,
		flags: u32,
		user_data: *mut (),
	) -> Result<()> {
		nix::ioctl_readwrite!(func, 'd', 0xB0, CrtcPageFlip);

		let mut ret = std::mem::MaybeUninit::<CrtcPageFlip>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).crtc_id = crtc_id;
			(*ptr).fb_id = fb_id;
			(*ptr).flags = flags;
			(*ptr).user_data = user_data as *const _ as _;

			func(self.file.as_raw_fd(), ptr)?;
		}

		Ok(())
	}
}

#[repr(C)]
pub struct Card {
	fb_id_ptr: u64,
	crtc_id_ptr: u64,
	connected_id_ptr: u64,
	encoder_id_ptr: u64,
	count_fbs: u32,
	count_crtcs: u32,
	count_connectors: u32,
	count_encoders: u32,
	min_width: u32,
	max_width: u32,
	min_height: u32,
	max_height: u32,
}

impl Card {
	pub fn fb_ids(&self) -> &[u32] {
		if self.count_fbs > 0 {
			unsafe { std::slice::from_raw_parts(self.fb_id_ptr as _, self.count_fbs as _) }
		} else {
			&[]
		}
	}

	pub fn crtc_ids(&self) -> &[u32] {
		if self.count_crtcs > 0 {
			unsafe { std::slice::from_raw_parts(self.crtc_id_ptr as _, self.count_crtcs as _) }
		} else {
			&[]
		}
	}

	pub fn connector_ids(&self) -> &[u32] {
		if self.count_connectors > 0 {
			unsafe {
				std::slice::from_raw_parts(self.connected_id_ptr as _, self.count_connectors as _)
			}
		} else {
			&[]
		}
	}

	pub fn encoder_ids(&self) -> &[u32] {
		if self.count_encoders > 0 {
			unsafe {
				std::slice::from_raw_parts(self.encoder_id_ptr as _, self.count_encoders as _)
			}
		} else {
			&[]
		}
	}
}

#[derive(Debug)]
#[repr(C)]
pub struct Connector {
	pub encoders_ptr: u64,
	pub modes_ptr: u64,
	pub props_ptr: u64,
	pub prop_values_ptr: u64,
	pub count_modes: u32,
	pub count_props: u32,
	pub count_encoders: u32,
	pub encoder_id: u32,
	pub connector_id: u32,
	pub connector_type: u32,
	pub connector_type_id: u32,
	pub connection: u32,
	pub mm_width: u32,
	pub mm_height: u32,
	pub subpixel: u32,
	pub pad: u32,
}

impl Connector {
	pub fn modes(&self) -> &[ModeInfo] {
		if self.count_modes != 0 {
			unsafe { std::slice::from_raw_parts(self.modes_ptr as _, self.count_modes as _) }
		} else {
			&[]
		}
	}
}

#[derive(Debug)]
#[repr(C)]
pub struct Encoder {
	pub encoder_id: u32,
	pub encoder_type: u32,
	pub crtc_id: u32,
	pub possible_crtcs: u32,
	pub possible_clones: u32,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ModeInfo {
	pub clock: u32,
	pub hdisplay: u16,
	pub hsync_start: u16,
	pub hsync_end: u16,
	pub htotal: u16,
	pub hskew: u16,
	pub vdisplay: u16,
	pub vsync_start: u16,
	pub vsync_end: u16,
	pub vtotal: u16,
	pub vscan: u16,
	pub vrefresh: u32,
	pub flags: u32,
	pub typee: u32,
	pub name: [u8; 32],
}

#[repr(C)]
pub struct FbCmd {
	pub fb_id: u32,
	pub width: u32,
	pub height: u32,
	pub pitch: u32,
	pub bpp: u32,
	pub depth: u32,
	pub handle: u32,
}

#[repr(C)]
pub struct Crtc {
	set_connectors_ptr: u64,
	count_connectors: u32,
	crtc_id: u32,
	fb_id: u32,
	x: u32,
	y: u32,
	gamma_size: u32,
	mode_valid: u32,
	mode: ModeInfo,
}

#[repr(C)]
pub struct CrtcPageFlip {
	crtc_id: u32,
	fb_id: u32,
	flags: u32,
	reserved: u32,
	user_data: u64,
}
