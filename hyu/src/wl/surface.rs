use glow::HasContext;

use crate::{state, wl, Point, Result};

#[derive(Default)]
pub struct SurfaceState {
	pub buffer: Option<wl::Buffer>,
	pub frame_callbacks: Vec<wl::Id<wl::Callback>>,
	pub input_region: Option<wl::Region>,
	pub presentation_feedback: Option<wl::Id<wl::WpPresentationFeedback>>,
}

impl SurfaceState {
	pub fn apply_to(&mut self, other: &mut Self) {
		if let Some(buffer) = std::mem::take(&mut self.buffer) {
			other.buffer = Some(buffer);
		}

		other.frame_callbacks.extend(&self.frame_callbacks);
		self.frame_callbacks.clear();

		if let Some(region) = std::mem::take(&mut self.input_region) {
			other.input_region = Some(region);
		}

		if let Some(presentation_feedback) = std::mem::take(&mut self.presentation_feedback) {
			other.presentation_feedback = Some(presentation_feedback);
		}
	}
}

pub enum SubSurfaceMode {
	Sync,
	Desync,
}

pub enum SurfaceRole {
	XdgToplevel,
	XdgPopup,
	SubSurface {
		mode: SubSurfaceMode,
		parent: wl::Id<wl::Surface>,
	},
	Cursor,
}

pub enum SurfaceTexture {
	Gl(glow::NativeTexture),
}

pub struct Surface {
	pub object_id: wl::Id<Self>,
	pub children: Vec<wl::Id<wl::SubSurface>>,
	pub data: Option<(Point, SurfaceTexture)>,
	pub role: Option<SurfaceRole>,
	pub pending: SurfaceState,
	pub current: SurfaceState,
}

impl Surface {
	pub fn new(object_id: wl::Id<Self>) -> Self {
		Self {
			object_id,
			children: Vec::new(),
			data: None,
			role: None,
			pending: Default::default(),
			current: Default::default(),
		}
	}

	pub fn push(&mut self, child: wl::Id<wl::SubSurface>) {
		self.children.push(child);
	}

	pub fn get_front_buffers(
		&self,
		client: &wl::Client,
	) -> Vec<(Point, Point, wl::Id<wl::Surface>)> {
		let Some(data) = self.data.as_ref() else {
			return Vec::new();
		};

		let mut ret = Vec::new();
		ret.push((Point(0, 0), data.0, self.object_id));

		for i in &self.children {
			let sub_surface = client.get_object(*i).unwrap();
			let surface = client.get_object(sub_surface.surface).unwrap();

			let position = sub_surface.position;

			ret.extend(
				surface
					.get_front_buffers(client)
					.into_iter()
					.map(|x| (x.0 + position, x.1, x.2)),
			);
		}

		ret
	}

	pub fn frame(&mut self, ms: u32, client: &mut wl::Client) -> Result<()> {
		for &callback in &self.current.frame_callbacks {
			let callback = client.get_object(callback)?.clone();
			callback.done(client, ms)?;
		}

		self.current.frame_callbacks.clear();

		for i in &self.children {
			let sub_surface = client.get_object(*i)?;
			let surface = client.get_object_mut(sub_surface.surface)?;

			surface.frame(ms, client)?;
		}

		Ok(())
	}

	pub fn presentation_feedback(
		&mut self,
		time: std::time::Duration,
		refresh: u32,
		sequence: u64,
		flags: u32,
		client: &mut wl::Client,
	) -> Result<()> {
		if let Some(presentation_feedback) = self.current.presentation_feedback {
			let presentation_feedback = client.get_object(presentation_feedback)?;

			let output = client
				.objects_mut::<wl::Output>()
				.first()
				.unwrap()
				.object_id;

			presentation_feedback.sync_output(client, output)?;
			presentation_feedback.presented(client, time, refresh, sequence, flags)?;

			self.current.presentation_feedback = None;
		}

		for &child in &self.children {
			let sub_surface = client.get_object(child)?;
			let surface = client.get_object_mut(sub_surface.surface)?;

			surface.presentation_feedback(time, refresh, sequence, flags, client)?;
		}

		Ok(())
	}

	pub fn gl_do_textures(&mut self, client: &mut wl::Client, glow: &glow::Context) -> Result<()> {
		if let Some(buffer) = &self.current.buffer {
			if let Some((size, tex)) = &self.data {
				if buffer.size != *size {
					let SurfaceTexture::Gl(tex) = tex;

					unsafe {
						glow.delete_texture(*tex);
					}

					self.data = None;
				}
			}

			let (_, texture) = self.data.get_or_insert_with(|| {
				let texture = unsafe { glow.create_texture().unwrap() };
				(buffer.size, SurfaceTexture::Gl(texture))
			});

			let SurfaceTexture::Gl(texture) = texture;

			buffer.gl_get_pixels(client, glow, *texture)?;

			buffer.release(client)?;
			self.current.buffer = None;
		}

		for i in &self.children {
			let sub_surface = client.get_object(*i)?;
			let surface = client.get_object_mut(sub_surface.surface)?;

			surface.gl_do_textures(client, glow)?;
		}

		Ok(())
	}

	pub fn set_role(&mut self, role: SurfaceRole) -> Result<()> {
		if let Some(old_role) = &self.role {
			if std::mem::discriminant(old_role) == std::mem::discriminant(&role) {
				return Ok(());
			}

			color_eyre::eyre::bail!("surface '{}' already has a role", *self.object_id);
		}

		self.role = Some(role);
		Ok(())
	}

	// https://wayland.app/protocols/wayland#wl_surface:request:commit
	pub fn commit(&mut self, client: &mut wl::Client) -> Result<()> {
		self.pending.apply_to(&mut self.current);

		if self.current.buffer.is_some() {
			self.gl_do_textures(client, &crate::backend::gl::GLOW)?;
		}

		client.render_tx.send(())?;
		Ok(())
	}
}

impl wl::Object for Surface {
	fn handle(&mut self, client: &mut wl::Client, op: u16, params: &[u8]) -> Result<()> {
		match op {
			0 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:destroy
				client
					.changes
					.push(state::Change::RemoveSurface(client.fd, self.object_id));

				unsafe {
					client.remove_object(self.object_id)?;
				}
			}
			1 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:attach
				let (buffer, x, y): (wl::Id<wl::Buffer>, u32, u32) =
					wlm::decode::from_slice(params)?;

				assert!(x == 0);
				assert!(y == 0);

				self.pending.buffer = if !buffer.is_null() {
					let buffer = client.get_object(buffer)?;
					Some(buffer.clone())
				} else {
					None
				};
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:damage
			}
			3 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:frame
				let callback: wl::Id<wl::Callback> = wlm::decode::from_slice(params)?;
				client.new_object(callback, wl::Callback::new(callback));

				self.pending.frame_callbacks.push(callback);
			}
			4 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_opaque_region
			}
			5 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_input_region
				let region: wl::Id<wl::Region> = wlm::decode::from_slice(params)?;

				let region = if region.is_null() {
					wl::Region::new(wl::Id::null())
				} else {
					client.get_object(region)?.clone()
				};

				self.pending.input_region = Some(region);
			}
			6 => {
				self.commit(client)?;
			}
			7 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_buffer_transform
			}
			8 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_buffer_scale
				let _scale: u32 = wlm::decode::from_slice(params)?;
			}
			9 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:damage_buffer
				let (_x, _y, _width, _height): (u32, u32, u32, u32) =
					wlm::decode::from_slice(params)?;
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Surface"),
		}

		Ok(())
	}
}
