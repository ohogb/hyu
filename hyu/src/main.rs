#![feature(unix_socket_ancillary_data)]
#![feature(generic_arg_infer)]
#![feature(seek_stream_len)]
#![feature(sync_unsafe_cell)]

pub mod backend;
pub mod egl;
mod global_wrapper;
mod point;
pub mod rt;
mod state;
mod stream;
pub mod tty;
pub mod wl;
pub mod xkb;

pub use global_wrapper::*;
pub use point::*;
pub use stream::*;

use clap::Parser as _;

use wl::Object as _;

use std::os::fd::AsRawFd as _;

pub type Result<T> = color_eyre::Result<T>;

#[derive(clap::Parser)]
struct Args {
	#[arg(short, long)]
	keymap: Option<String>,

	#[arg(short, long)]
	card: Option<std::path::PathBuf>,
}

struct Defer<T: FnMut()>(T);

impl<T: FnMut()> Drop for Defer<T> {
	fn drop(&mut self) {
		self.0()
	}
}

fn main() -> Result<()> {
	color_eyre::install()?;

	let args = Args::parse();

	let keymap = args.keymap.unwrap_or_default();
	let card = std::sync::Arc::from(
		args.card
			.as_ref()
			.map(|x| x.as_path())
			.unwrap_or_else(|| std::path::Path::new("/dev/dri/card0")),
	);

	let tty = tty::Device::open_current()?;

	tty.set_mode(1)?;
	let old_keyboard_mode = tty.get_keyboard_mode()?;
	tty.set_keyboard_mode(4)?;

	let _restorer = Defer(|| {
		let _ = tty.set_keyboard_mode(old_keyboard_mode);
		let _ = tty.set_mode(0);
	});

	let runtime_dir = std::env::var("XDG_RUNTIME_DIR")?;

	let index = 1;
	let path = std::path::PathBuf::from_iter([runtime_dir, format!("wayland-{index}")]);

	if path.exists() {
		std::fs::remove_file(&path)?;
	}

	let drm_state = backend::drm::initialize_state(&card)?;

	let render_tx = drm_state.render_tx.clone();

	let mut state = state::State {
		drm: drm_state,
		input: backend::input::initialize_state()?,
		compositor: state::CompositorState::new(render_tx),
	};

	state.compositor.initialize_xkb_state(keymap)?;

	let socket = std::os::unix::net::UnixListener::bind(&path)?;
	socket.set_nonblocking(true)?;

	let mut runtime = rt::Runtime::new();

	backend::drm::attach(&mut runtime, &mut state)?;
	backend::input::attach(&mut runtime, &mut state)?;

	runtime.on(
		rt::producers::UnixListener::new(socket),
		move |(stream, _), state, runtime| {
			stream.set_nonblocking(true)?;

			let stream = Stream::new(stream);
			let fd = stream.get().as_raw_fd();

			let mut client =
				wl::Client::new(fd, Point(0, 0), stream.clone(), state.drm.render_tx.clone());

			let mut display = wl::Display::new(wl::Id::new(1));

			display.push_global(wl::Shm::new(wl::Id::null()));
			display.push_global(wl::Compositor::new());
			display.push_global(wl::SubCompositor::new(wl::Id::null()));
			display.push_global(wl::DataDeviceManager::new());
			display.push_global(wl::Seat::new(
				wl::Id::null(),
				state.compositor.get_xkb_keymap(),
			));
			display.push_global(wl::Output::new(wl::Id::null()));
			display.push_global(wl::XdgWmBase::new(wl::Id::null()));
			display.push_global(wl::ZwpLinuxDmabufV1::new(wl::Id::null(), card.clone())?);
			display.push_global(wl::WpPresentation::new(wl::Id::null()));

			client.ensure_objects_capacity();
			client.new_object(wl::Id::new(1), display);

			state.compositor.clients.insert(fd, client);

			runtime.on(
				rt::producers::Wl::new(stream),
				move |msg, state, _| match msg {
					rt::producers::WlMessage::Request {
						object,
						op,
						params,
						fds,
					} => {
						let client = state.compositor.clients.get_mut(&fd).unwrap();
						client.received_fds.extend(fds);

						client.ensure_objects_capacity();

						let Some(object) = client.get_resource_mut(object) else {
							color_eyre::eyre::bail!("unknown object '{object}'");
						};

						object.handle(client, op, &params)?;

						state
							.compositor
							.changes
							.extend(std::mem::take(&mut client.changes));

						state.compositor.process_focus_changes()
					}
					rt::producers::WlMessage::Closed => {
						state
							.compositor
							.changes
							.push(state::Change::RemoveClient(fd));

						state.compositor.process_focus_changes()
					}
				},
			);

			Ok(())
		},
	);

	runtime.run(&mut state)?;

	drop(runtime);
	std::fs::remove_file(path)?;

	tty.set_keyboard_mode(old_keyboard_mode)?;
	tty.set_mode(0)?;

	Ok(())
}
