#![feature(unix_socket_ancillary_data)]
#![feature(generic_arg_infer)]
#![feature(seek_stream_len)]
#![feature(sync_unsafe_cell)]

pub mod backend;
mod client;
pub mod config;
pub mod drm;
pub mod elp;
pub mod gbm;
pub mod libinput;
mod point;
pub mod renderer;
mod state;
pub mod store;
mod stream;
pub mod tty;
pub mod udev;
pub mod wl;
pub mod xkb;

pub use client::*;
pub use config::*;
pub use point::*;
pub use store::*;
pub use stream::*;

use wl::Object as _;

use std::os::fd::AsRawFd as _;

pub type Result<T> = color_eyre::Result<T>;

struct Defer<T: FnMut()>(T);

impl<T: FnMut()> Drop for Defer<T> {
	fn drop(&mut self) {
		self.0()
	}
}

fn main() -> Result<()> {
	color_eyre::install()?;

	let config = Config::read_from_config_file()?;
	let tty = tty::Device::open_current()?;

	let old_keyboard_mode = tty.get_keyboard_mode()?;
	tty.set_mode(1)?;
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

	let drm_state = backend::drm::initialize_state(&config)?;

	let width = drm_state.screen.mode.hdisplay;
	let height = drm_state.screen.mode.vdisplay;

	let mut state = state::State {
		hw: state::HwState {
			drm: drm_state,
			input: backend::input::initialize_state()?,
		},
		compositor: state::CompositorState::create(width, height, &config)?,
	};

	let socket = std::os::unix::net::UnixListener::bind(&path)?;
	socket.set_nonblocking(true)?;

	let mut event_loop = elp::EventLoop::create()?;

	backend::drm::attach(&mut event_loop, &mut state)?;
	backend::input::attach(&mut event_loop, &mut state)?;

	event_loop.on(
		elp::unix_listener::create(socket),
		move |(stream, _), state, runtime| {
			stream.set_nonblocking(true)?;

			let stream = Stream::new(stream);
			let fd = stream.get().as_raw_fd();

			let mut client = Client::new(fd, Point(0, 0), stream.clone());

			let mut display = wl::Display::new(wl::Id::new(1));

			display.push_global(wl::Shm::new(wl::Id::null()));
			display.push_global(wl::Compositor::new());
			display.push_global(wl::SubCompositor::new(wl::Id::null()));
			display.push_global(wl::DataDeviceManager::new());
			display.push_global(wl::Seat::new(
				wl::Id::null(),
				state.compositor.xkb_state.keymap_file,
			));
			display.push_global(wl::Output::new(wl::Id::null()));
			display.push_global(wl::XdgWmBase::new(wl::Id::null()));
			display.push_global(wl::ZwpLinuxDmabufV1::new(wl::Id::null(), config)?);
			display.push_global(wl::WpPresentation::new(wl::Id::null()));
			display.push_global(wl::ZwlrLayerShellV1::new(wl::Id::null()));
			display.push_global(wl::ZxdgOutputManagerV1::new(wl::Id::null(), u32::MAX));

			client.ensure_objects_capacity();
			client.new_object(wl::Id::new(1), display);

			state.compositor.clients.insert(fd, client);

			runtime.on(elp::wl::create(stream), move |msg, state, _| match msg {
				elp::wl::Message::Request {
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

					object.handle(client, &mut state.hw, op, params)?;

					state
						.compositor
						.changes
						.extend(std::mem::take(&mut client.changes));

					state.compositor.process_focus_changes()
				}
				elp::wl::Message::Closed => {
					state
						.compositor
						.changes
						.push(state::Change::RemoveClient(fd));

					state.compositor.process_focus_changes()
				}
			})
		},
	)?;

	event_loop.run(&mut state)?;

	drop(event_loop);
	std::fs::remove_file(path)?;

	Ok(())
}
