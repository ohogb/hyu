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

	pub fn add_fb2(
		&self,
		width: u32,
		height: u32,
		format: u32,
		handles: [u32; 4],
		pitches: [u32; 4],
		offsets: [u32; 4],
		modifiers: [u64; 4],
	) -> Result<u32> {
		nix::ioctl_readwrite!(func, 'd', 0xB8, FbCmd2);

		let mut ret = std::mem::MaybeUninit::<FbCmd2>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).width = width;
			(*ptr).height = height;
			(*ptr).pixel_format = format;
			(*ptr).flags = 1 << 1;
			(*ptr).handles = handles;
			(*ptr).pitches = pitches;
			(*ptr).offsets = offsets;
			(*ptr).modifier = modifiers;

			func(self.file.as_raw_fd(), ptr)?;
		}

		Ok(unsafe { ret.assume_init() }.fb_id)
	}

	pub fn get_crtc(&self, crtc_id: u32) -> Result<Crtc> {
		nix::ioctl_readwrite!(func, 'd', 0xA1, Crtc);

		let mut ret = std::mem::MaybeUninit::<Crtc>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).crtc_id = crtc_id;

			func(self.file.as_raw_fd(), ptr).unwrap();
		};

		Ok(unsafe { ret.assume_init() })
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

	pub fn get_plane_resources(&self) -> Result<PlaneResources> {
		nix::ioctl_readwrite!(func, 'd', 0xB5, PlaneResources);
		let mut ret = std::mem::MaybeUninit::<PlaneResources>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();

			func(self.file.as_raw_fd(), ptr).unwrap();

			if (*ptr).count_planes > 0 {
				(*ptr).plane_id_ptr =
					Box::leak(vec![0u32; (*ptr).count_planes as _].into_boxed_slice()).as_mut_ptr()
						as u64;
			}

			func(self.file.as_raw_fd(), ptr).unwrap();
		};

		Ok(unsafe { ret.assume_init() })
	}

	pub fn get_plane(&self, plane_id: u32) -> Result<Plane> {
		nix::ioctl_readwrite!(func, 'd', 0xB6, Plane);
		let mut ret = std::mem::MaybeUninit::<Plane>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).plane_id = plane_id;

			func(self.file.as_raw_fd(), ptr).unwrap();

			if (*ptr).count_format_types > 0 {
				(*ptr).format_type_ptr =
					Box::leak(vec![0u32; (*ptr).count_format_types as _].into_boxed_slice())
						.as_mut_ptr() as u64;
			}

			func(self.file.as_raw_fd(), ptr).unwrap();
		};

		Ok(unsafe { ret.assume_init() })
	}

	pub fn get_props(&self, object_id: u32, object_type: u32) -> Result<Props> {
		nix::ioctl_readwrite!(func, 'd', 0xB9, Props);

		let mut ret = std::mem::MaybeUninit::<Props>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).obj_id = object_id;
			(*ptr).obj_type = object_type;

			func(self.file.as_raw_fd(), ptr).unwrap();

			if (*ptr).count_props > 0 {
				(*ptr).props_ptr = Box::leak(vec![0u32; (*ptr).count_props as _].into_boxed_slice())
					.as_mut_ptr() as u64;

				(*ptr).prop_values_ptr =
					Box::leak(vec![0u64; (*ptr).count_props as _].into_boxed_slice()).as_mut_ptr()
						as u64;
			}

			func(self.file.as_raw_fd(), ptr).unwrap();
		};

		Ok(unsafe { ret.assume_init() })
	}

	pub fn get_prop(&self, prop_id: u32) -> Result<Prop> {
		nix::ioctl_readwrite!(func, 'd', 0xAA, Prop);

		let mut ret = std::mem::MaybeUninit::<Prop>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).prop_id = prop_id;

			func(self.file.as_raw_fd(), ptr).unwrap();

			if (*ptr).count_values > 0 {
				(*ptr).values_ptr =
					Box::leak(vec![0u64; (*ptr).count_values as _].into_boxed_slice()).as_mut_ptr()
						as u64;
			}

			// TODO: enums blobs
			(*ptr).count_enum_blobs = 0;

			func(self.file.as_raw_fd(), ptr).unwrap();
		};

		Ok(unsafe { ret.assume_init() })
	}

	pub fn create_blob(&self, data: &[u8]) -> Result<u32> {
		nix::ioctl_readwrite!(func, 'd', 0xBD, Blob);

		let mut ret = std::mem::MaybeUninit::<Blob>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).data = data.as_ptr() as _;
			(*ptr).length = data.len() as _;

			func(self.file.as_raw_fd(), ptr).unwrap();
		};

		Ok(unsafe { ret.assume_init().blob_id })
	}

	pub fn begin_atomic(&self) -> AtomicHelper {
		AtomicHelper { props: Vec::new() }
	}

	pub fn commit(&self, ctx: &AtomicHelper, flags: u32, user_data: *mut ()) -> Result<()> {
		nix::ioctl_readwrite!(func, 'd', 0xBC, AtomicCommit);

		let mut ret = std::mem::MaybeUninit::<AtomicCommit>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();

			let mut objs = Vec::new();
			let mut prop_counts = Vec::new();
			let mut old = u32::MAX;

			for &(obj, ..) in &ctx.props {
				if obj != old {
					objs.push(obj);
					prop_counts.push(0);

					old = obj;
				}

				*prop_counts.last_mut().unwrap() += 1;
			}

			let props = ctx.props.iter().map(|x| x.1).collect::<Vec<_>>();
			let prop_values = ctx.props.iter().map(|x| x.2).collect::<Vec<_>>();

			(*ptr).flags = flags;
			(*ptr).user_data = user_data as _;

			(*ptr).count_objs = objs.len() as _;
			(*ptr).objs_ptr = objs.as_ptr() as _;
			(*ptr).count_props_ptr = prop_counts.as_ptr() as _;
			(*ptr).props_ptr = props.as_ptr() as _;
			(*ptr).prop_values_ptr = prop_values.as_ptr() as _;

			func(self.file.as_raw_fd(), ptr)?;
		}

		Ok(())
	}

	pub fn set_client_capability(&self, capability: u64, value: u64) -> Result<()> {
		nix::ioctl_readwrite!(func, 'd', 0x0D, ClientCapability);

		let mut ret = std::mem::MaybeUninit::<ClientCapability>::zeroed();
		unsafe {
			let ptr = ret.as_mut_ptr();
			(*ptr).capability = capability;
			(*ptr).value = value;

			func(self.file.as_raw_fd(), ptr).unwrap();
		};

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

#[derive(Debug, Clone)]
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

impl HasProps for Connector {
	fn get_props(&self, device: &Device) -> Result<Props> {
		device.get_props(self.connector_id, 0xC0C0C0C0)
	}
}

impl Object for Connector {
	fn get_id(&self) -> u32 {
		self.connector_id
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
pub struct FbCmd2 {
	pub fb_id: u32,
	pub width: u32,
	pub height: u32,
	pub pixel_format: u32,
	pub flags: u32,
	pub handles: [u32; 4],
	pub pitches: [u32; 4],
	pub offsets: [u32; 4],
	pub modifier: [u64; 4],
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

impl HasProps for Crtc {
	fn get_props(&self, device: &Device) -> Result<Props> {
		device.get_props(self.crtc_id, 0xCCCCCCCC)
	}
}

impl Object for Crtc {
	fn get_id(&self) -> u32 {
		self.crtc_id
	}
}

#[repr(C)]
pub struct CrtcPageFlip {
	crtc_id: u32,
	fb_id: u32,
	flags: u32,
	reserved: u32,
	user_data: u64,
}

#[derive(Debug)]
#[repr(C)]
pub struct PlaneResources {
	plane_id_ptr: u64,
	count_planes: u32,
}

impl PlaneResources {
	pub fn plane_ids(&self) -> &[u32] {
		if self.count_planes > 0 {
			unsafe { std::slice::from_raw_parts(self.plane_id_ptr as _, self.count_planes as _) }
		} else {
			&[]
		}
	}
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Plane {
	pub plane_id: u32,
	pub crtc_id: u32,
	pub fb_id: u32,
	pub possible_crtcs: u32,
	pub gamma_size: u32,
	pub count_format_types: u32,
	pub format_type_ptr: u64,
}

impl HasProps for Plane {
	fn get_props(&self, device: &Device) -> Result<Props> {
		device.get_props(self.plane_id, 0xEEEEEEEE)
	}
}

impl Object for Plane {
	fn get_id(&self) -> u32 {
		self.plane_id
	}
}

#[repr(C)]
pub struct Props {
	props_ptr: u64,
	prop_values_ptr: u64,
	count_props: u32,
	obj_id: u32,
	obj_type: u32,
}

impl Props {
	pub fn prop_ids(&self) -> &[u32] {
		if self.count_props > 0 {
			unsafe { std::slice::from_raw_parts(self.props_ptr as _, self.count_props as _) }
		} else {
			&[]
		}
	}

	pub fn prop_values(&self) -> &[u64] {
		if self.count_props > 0 {
			unsafe { std::slice::from_raw_parts(self.prop_values_ptr as _, self.count_props as _) }
		} else {
			&[]
		}
	}
}

#[repr(C)]
pub struct Prop {
	pub values_ptr: u64,
	pub enum_blob_ptr: u64,
	pub prop_id: u32,
	pub flags: u32,
	pub name: [u8; 0x20],
	pub count_values: u32,
	pub count_enum_blobs: u32,
}

pub struct PropWrapper<T: HasProps + Object> {
	props: Props,
	prop_info: std::collections::HashMap<u32, Prop>,
	object: T,
}

impl<T: HasProps + Object> PropWrapper<T> {
	pub fn new(object: T, device: &Device) -> Self {
		let props = object.get_props(device).unwrap();
		let prop_info = props
			.prop_ids()
			.iter()
			.map(|&x| (x, device.get_prop(x).unwrap()));

		let prop_info = std::collections::HashMap::from_iter(prop_info);

		Self {
			props,
			prop_info,
			object,
		}
	}

	pub fn find_property(&self, property: impl AsRef<str>) -> Option<u32> {
		self.props
			.prop_ids()
			.iter()
			.find(|&x| {
				let prop = self.prop_info.get(x).unwrap();
				let name = unsafe { std::ffi::CStr::from_ptr(prop.name.as_ptr() as _) }
					.to_str()
					.unwrap();

				name == property.as_ref()
			})
			.copied()
	}
}

impl<T: HasProps + Object> std::ops::Deref for PropWrapper<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.object
	}
}

pub trait HasProps {
	fn get_props(&self, device: &Device) -> Result<Props>;
}

pub trait Object {
	fn get_id(&self) -> u32;
}

pub struct AtomicHelper {
	pub props: Vec<(u32, u32, u64)>,
}

impl AtomicHelper {
	pub fn add_property(
		&mut self,
		object: &PropWrapper<impl HasProps + Object>,
		property_id: u32,
		value: u64,
	) {
		self.props.push((object.get_id(), property_id, value));
	}

	pub fn clear(&mut self) {
		self.props.clear();
	}
}

#[repr(C)]
pub struct Blob {
	data: u64,
	length: u32,
	blob_id: u32,
}

#[repr(C)]
pub struct AtomicCommit {
	flags: u32,
	count_objs: u32,
	objs_ptr: u64,
	count_props_ptr: u64,
	props_ptr: u64,
	prop_values_ptr: u64,
	reserved: u64,
	user_data: u64,
}

#[repr(C)]
pub struct ClientCapability {
	capability: u64,
	value: u64,
}
