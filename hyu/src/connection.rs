use std::os::fd::AsRawFd as _;

use crate::Result;

pub struct Connection {
	stream: std::os::unix::net::UnixStream,
}

impl Connection {
	pub fn new(stream: std::os::unix::net::UnixStream) -> Self {
		Self { stream }
	}

	pub fn as_raw_fd(&self) -> std::os::fd::RawFd {
		self.stream.as_raw_fd()
	}

	pub fn send_message<T: serde::Serialize>(&self, message: wlm::Message<T>) -> Result<()> {
		let mut cmsg_buffer = [0u8; 0x20];
		let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

		let ret = self
			.stream
			.send_vectored_with_ancillary(&[std::io::IoSlice::new(&message.to_vec()?)], &mut cmsg);

		if ret.is_err() {
			eprintln!("Client::send_message() failed!");
		}

		Ok(())
	}

	pub fn send_message_with_fd<T: serde::Serialize>(
		&self,
		message: wlm::Message<T>,
		fd: std::os::fd::RawFd,
	) -> Result<()> {
		let mut cmsg_buffer = [0u8; 0x20];
		let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

		cmsg.add_fds(&[fd]);

		let ret = self
			.stream
			.send_vectored_with_ancillary(&[std::io::IoSlice::new(&message.to_vec()?)], &mut cmsg);

		if ret.is_err() {
			eprintln!("Client::send_message() failed!");
		}

		Ok(())
	}

	pub fn read(&self, n: usize) -> Result<Option<(Vec<u8>, Vec<std::os::fd::RawFd>)>> {
		let mut cmsg_buffer = [0u8; 0x40];
		let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

		let mut bytes = vec![0u8; n];

		let len = self
			.stream
			.recv_vectored_with_ancillary(&mut [std::io::IoSliceMut::new(&mut bytes)], &mut cmsg);

		let len = match len {
			Ok(len) => len,
			Err(x) => match x.kind() {
				std::io::ErrorKind::ConnectionReset => {
					return Ok(None);
				}
				_ => {
					return Err(x)?;
				}
			},
		};

		if len == 0 {
			return Ok(None);
		}

		if len != n {
			color_eyre::eyre::bail!(
				"socket did not have enough bytes to read, expected {n}, got {len}"
			);
		}

		let mut fds = Vec::new();

		for i in cmsg.messages() {
			let std::os::unix::net::AncillaryData::ScmRights(scm_rights) = i.unwrap() else {
				continue;
			};

			fds.extend(scm_rights.into_iter());
		}

		Ok(Some((bytes, fds)))
	}
}
