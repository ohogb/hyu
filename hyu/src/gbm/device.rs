use crate::{gbm, wl};

#[link(name = "gbm")]
unsafe extern "C" {
	fn gbm_bo_create_with_modifiers2(
		device: usize,
		width: u32,
		height: u32,
		format: u32,
		modifiers: usize,
		count: u32,
		flags: u32,
	) -> Option<gbm::BufferObject>;
	fn gbm_bo_import(
		device: usize,
		r#type: u32,
		buffer: usize,
		flags: u32,
	) -> Option<gbm::BufferObject>;
	fn gbm_create_device(fd: std::os::fd::RawFd) -> Option<Device>;
	fn gbm_device_destroy(device: usize);
}

const GBM_BO_IMPORT_FD_MODIFIER: u32 = 0x5504;

#[repr(C)]
struct GbmImportFdModifierData {
	width: u32,
	height: u32,
	format: u32,
	num_fds: u32,
	fds: [i32; 4],
	strides: [i32; 4],
	offsets: [i32; 4],
	modifier: u64,
}

#[repr(transparent)]
pub struct Device {
	ptr: std::num::NonZeroUsize,
}

impl Device {
	pub fn create(fd: std::os::fd::RawFd) -> Option<Self> {
		unsafe { gbm_create_device(fd) }
	}

	pub fn as_ptr(&self) -> usize {
		self.ptr.get()
	}

	pub fn create_buffer_object(
		&self,
		width: u32,
		height: u32,
		format: u32,
		modifiers: &[u64],
		flags: u32,
	) -> Option<gbm::BufferObject> {
		unsafe {
			gbm_bo_create_with_modifiers2(
				self.as_ptr(),
				width,
				height,
				format,
				modifiers.as_ptr() as _,
				modifiers.len() as _,
				flags,
			)
		}
	}

	pub fn import_dmabuf(&self, attributes: &wl::DmabufAttributes) -> Option<gbm::BufferObject> {
		let mut data = GbmImportFdModifierData {
			width: attributes.width,
			height: attributes.height,
			format: 0x34325258,
			num_fds: attributes.planes.len() as _,
			fds: Default::default(),
			strides: Default::default(),
			offsets: Default::default(),
			modifier: attributes.modifier,
		};

		for (out, plane) in std::iter::zip(&mut data.fds, &attributes.planes) {
			*out = plane.fd;
		}

		for (out, plane) in std::iter::zip(&mut data.strides, &attributes.planes) {
			*out = plane.stride as _;
		}

		for (out, plane) in std::iter::zip(&mut data.offsets, &attributes.planes) {
			*out = plane.offset as _;
		}

		unsafe {
			gbm_bo_import(
				self.as_ptr(),
				GBM_BO_IMPORT_FD_MODIFIER,
				&raw const data as _,
				1 << 0,
			)
		}
	}
}

impl Drop for Device {
	fn drop(&mut self) {
		unsafe {
			gbm_device_destroy(self.as_ptr());
		}
	}
}
