use std::{io::Read as _, os::fd::AsRawFd as _};

use crate::{elp, Result};

pub struct Source {
	stream: crate::Stream,
	params: Vec<u8>,
}

pub enum Message<'a> {
	Request {
		object: u32,
		op: u16,
		params: &'a [u8],
		fds: &'a [std::os::fd::RawFd],
	},
	Closed,
}

impl elp::Source for Source {
	type Message<'a> = Message<'a>;
	type Ret = Result<()>;

	fn fd(&self) -> std::os::fd::RawFd {
		self.stream.get().as_raw_fd()
	}

	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>> {
		let mut cmsg_buffer = [0u8; 0x40];
		let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

		let mut obj = [0u8; 4];

		let len = self
			.stream
			.get()
			.recv_vectored_with_ancillary(&mut [std::io::IoSliceMut::new(&mut obj)], &mut cmsg);

		let len = match len {
			Ok(len) => len,
			Err(x) => match x.kind() {
				std::io::ErrorKind::ConnectionReset => {
					callback(Message::Closed)?;
					return Ok(std::ops::ControlFlow::Break(()));
				}
				_ => {
					return Err(x)?;
				}
			},
		};

		if len == 0 {
			callback(Message::Closed)?;
			return Ok(std::ops::ControlFlow::Break(()));
		}

		let mut fds = Vec::new();

		for i in cmsg.messages() {
			let std::os::unix::net::AncillaryData::ScmRights(scm_rights) = i.unwrap() else {
				continue;
			};

			fds.extend(scm_rights.into_iter());
		}

		let mut op = [0u8; 2];
		self.stream.get().read_exact(&mut op)?;

		let mut size = [0u8; 2];
		self.stream.get().read_exact(&mut size)?;

		let size = u16::from_ne_bytes(size) - 0x8;

		self.params.resize(size as _, 0);
		self.stream.get().read_exact(&mut self.params)?;

		let object = u32::from_ne_bytes(obj);
		let op = u16::from_ne_bytes(op);

		callback(Message::Request {
			object,
			op,
			params: &self.params,
			fds: &fds,
		})?;

		self.params.clear();
		Ok(std::ops::ControlFlow::Continue(()))
	}
}

pub fn create(stream: crate::Stream) -> Source {
	Source {
		stream,
		params: Vec::new(),
	}
}
