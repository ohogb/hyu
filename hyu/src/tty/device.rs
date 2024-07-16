use std::os::fd::{AsFd as _, AsRawFd as _};

use crate::Result;

pub struct Device {
	file: std::fs::File,
}

impl Device {
	pub fn open_current() -> Result<Self> {
		let path = nix::unistd::ttyname(std::io::stdin().as_fd())?;

		let tty = std::fs::OpenOptions::new()
			.read(true)
			.write(true)
			.open(&path)?;

		Ok(Self { file: tty })
	}

	pub fn get_keyboard_mode(&self) -> Result<u64> {
		let mut value = 0u64;
		let ret = unsafe { nix::libc::ioctl(self.fd(), 0x4B44, &mut value as *mut _) };

		if ret == 0 {
			Ok(value)
		} else {
			Err(std::io::Error::from_raw_os_error(ret))?
		}
	}

	pub fn set_keyboard_mode(&self, mode: u64) -> Result<()> {
		let ret = unsafe { nix::libc::ioctl(self.fd(), 0x4B45, mode) };

		if ret == 0 {
			Ok(())
		} else {
			Err(std::io::Error::from_raw_os_error(ret))?
		}
	}

	pub fn set_mode(&self, mode: u64) -> Result<()> {
		let ret = unsafe { nix::libc::ioctl(self.fd(), 0x4B3A, mode) };

		if ret == 0 {
			Ok(())
		} else {
			Err(std::io::Error::from_raw_os_error(ret))?
		}
	}

	fn fd(&self) -> std::os::fd::RawFd {
		self.file.as_raw_fd()
	}
}
