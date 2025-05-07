use std::rc::Rc;

use crate::{Connection, Result, elp};

pub struct Source {
	connection: Rc<Connection>,
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
		self.connection.as_raw_fd()
	}

	fn call(
		&mut self,
		callback: &mut impl FnMut(Self::Message<'_>) -> Self::Ret,
	) -> Result<std::ops::ControlFlow<()>> {
		const HEADER_SIZE: usize = 4 + 2 + 2;

		let Some((header, mut fds)) = self.connection.read(HEADER_SIZE)? else {
			callback(Message::Closed)?;
			return Ok(std::ops::ControlFlow::Break(()));
		};

		assert!(header.len() == HEADER_SIZE);

		let object = u32::from_ne_bytes(<[u8; 4]>::try_from(&header[0..4])?);
		let op = u16::from_ne_bytes(<[u8; 2]>::try_from(&header[4..6])?);
		let size = u16::from_ne_bytes(<[u8; 2]>::try_from(&header[6..8])?);

		assert!(size >= HEADER_SIZE as u16);
		let size = size - HEADER_SIZE as u16;

		let params = if size != 0 {
			let Some((params, second_fds)) = self.connection.read(size as _)? else {
				callback(Message::Closed)?;
				return Ok(std::ops::ControlFlow::Break(()));
			};

			assert!(params.len() == size as usize);
			fds.extend(second_fds);

			params
		} else {
			Vec::new()
		};

		callback(Message::Request {
			object,
			op,
			params: &params,
			fds: &fds,
		})?;

		Ok(std::ops::ControlFlow::Continue(()))
	}
}

pub fn create(connection: Rc<Connection>) -> Source {
	Source { connection }
}
