#![feature(unix_socket_ancillary_data)]
#![feature(generic_arg_infer)]
#![feature(seek_stream_len)]
#![feature(sync_unsafe_cell)]

pub mod backend;
pub mod egl;
mod global_wrapper;
mod point;
mod state;
mod stream;
pub mod tty;
pub mod wl;
pub mod xkb;

use clap::Parser as _;
pub use global_wrapper::*;
pub use point::*;
pub use stream::*;

use wl::Object;

use std::{io::Read, os::fd::AsRawFd};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn client_event_loop(stream: std::os::unix::net::UnixStream, index: usize) -> Result<()> {
	stream.set_nonblocking(true)?;

	let fd = stream.as_raw_fd();
	let stream = Stream::new(stream);

	let mut client = wl::Client::new(
		fd,
		Point((100 * index + 10) as i32, (100 * index + 10) as i32),
		stream.clone(),
	);

	let mut display = wl::Display::new(wl::Id::new(1));

	display.push_global(wl::Shm::new(wl::Id::null()));
	display.push_global(wl::Compositor::new());
	display.push_global(wl::SubCompositor::new(wl::Id::null()));
	display.push_global(wl::DataDeviceManager::new());
	display.push_global(wl::Seat::new(wl::Id::null(), state::get_xkb_keymap()));
	display.push_global(wl::Output::new(wl::Id::null()));
	display.push_global(wl::XdgWmBase::new(wl::Id::null()));
	display.push_global(wl::ZwpLinuxDmabufV1::new(wl::Id::null())?);
	display.push_global(wl::WpPresentation::new(wl::Id::null()));

	client.ensure_objects_capacity();
	client.new_object(wl::Id::new(1), display);

	state::CLIENTS.lock().unwrap().insert(fd, client);

	let mut params = Vec::new();

	loop {
		nix::poll::poll(
			&mut [nix::poll::PollFd::new(
				unsafe { std::os::fd::BorrowedFd::borrow_raw(fd) },
				nix::poll::PollFlags::POLLIN,
			)],
			nix::poll::PollTimeout::NONE,
		)?;

		let mut cmsg_buffer = [0u8; 0x40];
		let mut cmsg = std::os::unix::net::SocketAncillary::new(&mut cmsg_buffer);

		let mut obj = [0u8; 4];

		let len = stream
			.get()
			.recv_vectored_with_ancillary(&mut [std::io::IoSliceMut::new(&mut obj)], &mut cmsg);

		let mut clients = state::CLIENTS.lock().unwrap();

		let len = match len {
			Ok(len) => len,
			Err(x) => match x.kind() {
				std::io::ErrorKind::ConnectionReset => {
					state::CHANGES
						.lock()
						.unwrap()
						.push(state::Change::RemoveClient(fd));

					state::process_focus_changes(&mut clients)?;
					return Ok(());
				}
				_ => {
					return Err(x)?;
				}
			},
		};

		if len == 0 {
			state::CHANGES
				.lock()
				.unwrap()
				.push(state::Change::RemoveClient(fd));

			state::process_focus_changes(&mut clients)?;
			return Ok(());
		}

		let client = clients.get_mut(&fd).unwrap();

		for i in cmsg.messages() {
			let std::os::unix::net::AncillaryData::ScmRights(scm_rights) = i.unwrap() else {
				continue;
			};

			client.received_fds.extend(scm_rights.into_iter());
		}

		let mut op = [0u8; 2];
		stream.get().read_exact(&mut op)?;

		let mut size = [0u8; 2];
		stream.get().read_exact(&mut size)?;

		let size = u16::from_ne_bytes(size) - 0x8;

		params.resize(size as _, 0);
		stream.get().read_exact(&mut params)?;

		let object = u32::from_ne_bytes(obj);
		let op = u16::from_ne_bytes(op);

		client.ensure_objects_capacity();

		let Some(object) = client.get_resource_mut(object) else {
			return Err(format!("unknown object '{object}'"))?;
		};

		object.handle(client, op, &params)?;
		params.clear();

		state::process_focus_changes(&mut clients)?;
	}
}

#[derive(clap::Parser)]
struct Args {
	#[arg(short, long)]
	keymap: Option<String>,
}

fn main() -> Result<()> {
	let args = Args::parse();
	state::initialize_xkb_state(args.keymap.unwrap_or_default())?;

	let tty = tty::Device::open_current()?;

	std::thread::spawn(|| backend::drm::run().unwrap());
	std::thread::spawn(|| backend::input::run().unwrap());

	tty.set_mode(1)?;

	let old_keyboard_mode = tty.get_keyboard_mode()?;
	tty.set_keyboard_mode(4)?;

	// std::thread::spawn(|| backend::winit::run(backend::gl::Setup).unwrap());

	let runtime_dir = std::env::var("XDG_RUNTIME_DIR")?;

	let index = 1;
	let path = std::path::PathBuf::from_iter([runtime_dir, format!("wayland-{index}")]);

	if path.exists() {
		std::fs::remove_file(&path)?;
	}

	let socket = std::os::unix::net::UnixListener::bind(&path)?;
	socket.set_nonblocking(true)?;

	while !state::quit() {
		let (stream, _) = match socket.accept() {
			Ok(x) => x,
			Err(x) if x.kind() == std::io::ErrorKind::WouldBlock => {
				std::thread::sleep(std::time::Duration::from_millis(10));
				continue;
			}
			Err(x) => Err(x)?,
		};

		std::thread::spawn(move || client_event_loop(stream, 0).unwrap());
	}

	drop(socket);
	std::fs::remove_file(path)?;

	tty.set_keyboard_mode(old_keyboard_mode)?;
	tty.set_mode(0)?;

	Ok(())
}
