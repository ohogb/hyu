use glow::HasContext;

use crate::{
	Client, Point, Result,
	renderer::{self, gl},
	state::{self, HwState},
	wl,
};

#[derive(Default)]
pub struct SurfaceState {
	pub buffer: Option<wl::Id<wl::Buffer>>,
	pub frame_callbacks: Vec<wl::Id<wl::Callback>>,
	pub input_region: Option<wl::Region>,
	pub offset: Option<(i32, i32)>,
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

		if let Some(offset) = std::mem::take(&mut self.offset) {
			other.offset = Some(offset);
		}

		if let Some(presentation_feedback) = std::mem::take(&mut self.presentation_feedback) {
			other.presentation_feedback = Some(presentation_feedback);
		}
	}
}

pub enum SubSurfaceMode {
	Sync { state_to_apply: SurfaceState },
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
	LayerSurface {
		wlr_layer_surface: wl::Id<wl::ZwlrLayerSurfaceV1>,
		initial_commit: bool,
	},
}

pub enum SurfaceTexture {
	Gl(glow::NativeTexture),
	Vk(renderer::vulkan::Texture),
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
		client: &Client,
	) -> Result<Vec<(Point, Point, wl::Id<wl::Surface>)>> {
		let Some(data) = self.data.as_ref() else {
			return Ok(Vec::new());
		};

		let mut ret = Vec::new();
		ret.push((Point(0, 0), data.0, self.object_id));

		for i in &self.children {
			let sub_surface = client.get_object(*i)?;
			let surface = client.get_object(sub_surface.surface)?;

			let position = sub_surface.position;

			ret.extend(
				surface
					.get_front_buffers(client)?
					.into_iter()
					.map(|x| (x.0 + position, x.1, x.2)),
			);
		}

		Ok(ret)
	}

	pub fn frame(&mut self, ms: u32, client: &mut Client) -> Result<()> {
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
		till_next_refresh: std::time::Duration,
		sequence: u64,
		flags: u32,
		client: &mut Client,
	) -> Result<()> {
		if let Some(presentation_feedback) = self.current.presentation_feedback {
			let presentation_feedback = client.get_object(presentation_feedback)?;

			let output = client
				.objects_mut::<wl::Output>()
				.first()
				.unwrap()
				.object_id;

			presentation_feedback.sync_output(client, output)?;
			presentation_feedback.presented(client, time, till_next_refresh, sequence, flags)?;

			self.current.presentation_feedback = None;
		}

		for &child in &self.children {
			let sub_surface = client.get_object(child)?;
			let surface = client.get_object_mut(sub_surface.surface)?;

			surface.presentation_feedback(time, till_next_refresh, sequence, flags, client)?;
		}

		Ok(())
	}

	// pub fn gl_do_textures(&mut self, client: &mut Client, glow: &glow::Context) -> Result<()> {
	// 	if let Some(buffer) = &self.current.buffer {
	// 		let buffer = client.get_object(*buffer)?;
	//
	// 		if let Some((size, tex)) = &self.data {
	// 			if buffer.size != *size {
	// 				let SurfaceTexture::Gl(tex) = tex;
	//
	// 				unsafe {
	// 					glow.delete_texture(*tex);
	// 				}
	//
	// 				self.data = None;
	// 			}
	// 		}
	//
	// 		let (_, texture) = self.data.get_or_insert_with(|| {
	// 			let texture = unsafe { glow.create_texture().unwrap() };
	// 			(buffer.size, SurfaceTexture::Gl(texture))
	// 		});
	//
	// 		let SurfaceTexture::Gl(texture) = texture;
	//
	// 		buffer.gl_get_pixels(client, glow, *texture)?;
	//
	// 		buffer.release(client)?;
	// 		self.current.buffer = None;
	// 	}
	//
	// 	Ok(())
	// }

	pub fn vk_do_textures(
		&mut self,
		client: &mut Client,
		vk: &mut renderer::vulkan::Renderer,
	) -> Result<()> {
		if let Some(buffer) = &self.current.buffer {
			let buffer = client.get_object(*buffer)?;

			if let Some((size, tex)) = &self.data {
				if buffer.size != *size {
					// panic!("{:?} {:?}", buffer.size, *size);
					let SurfaceTexture::Vk(texture) = tex else {
						panic!();
					};

					vk.textures_to_delete.push(texture.clone());
					self.data = None;
				}
			}

			// let (_, texture) = self.data.get_or_insert_with(|| {
			//              let texture =
			// 	// let texture = unsafe { glow.create_texture().unwrap() };
			// 	(buffer.size, SurfaceTexture::Gl(texture))
			// });

			// let SurfaceTexture::Vk(texture) = texture else {
			// 	panic!();
			// };

			// buffer.gl_get_pixels(client, glow, *texture)?;
			buffer.vk_copy_to_texture(client, vk, &mut self.data)?;

			buffer.release(client)?;
			self.current.buffer = None;
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
	pub fn commit(&mut self, client: &mut Client, hw_state: &mut HwState) -> Result<()> {
		if let Some(SurfaceRole::LayerSurface {
			wlr_layer_surface,
			initial_commit,
		}) = &mut self.role
		{
			if *initial_commit {
				*initial_commit = false;

				let wl_display = client.get_object_mut(wl::Id::<wl::Display>::new(1))?;

				let wlr_layer_surface = client.get_object_mut(*wlr_layer_surface)?;
				wlr_layer_surface.configure(client, wl_display.new_serial(), 0, 0)?;

				return Ok(());
			}
		}

		let mut synced_sub_surface = false;

		if let Some(SurfaceRole::SubSurface {
			mode: SubSurfaceMode::Sync { state_to_apply },
			..
		}) = &mut self.role
		{
			self.pending.apply_to(state_to_apply);
			synced_sub_surface = true;
		}

		if synced_sub_surface {
			return Ok(());
		}

		self.pending.apply_to(&mut self.current);

		if self.current.buffer.is_some() {
			// self.gl_do_textures(client, &gl::GLOW)?;
			self.vk_do_textures(client, &mut hw_state.drm.vulkan)?;
		}

		self.depth_first_sub_tree(client, &mut |client, _, surface| {
			let Some(SurfaceRole::SubSurface { mode, .. }) = &mut surface.role else {
				panic!();
			};

			// TODO: check if parent should override mode
			if let SubSurfaceMode::Sync { state_to_apply } = mode {
				state_to_apply.apply_to(&mut surface.current);

				if surface.current.buffer.is_some() {
					surface.vk_do_textures(client, &mut hw_state.drm.vulkan)?;
					// surface.gl_do_textures(client, &gl::GLOW)?;
				}
			}

			Ok(())
		})?;

		Ok(())
	}

	pub fn depth_first_sub_tree(
		&self,
		client: &mut Client,
		callback: &mut impl FnMut(&mut Client, &mut wl::SubSurface, &mut wl::Surface) -> Result<()>,
	) -> Result<()> {
		for &child in &self.children {
			let sub_surface = client.get_object_mut(child)?;
			let surface = client.get_object_mut(sub_surface.surface)?;

			callback(client, sub_surface, surface)?;
			surface.depth_first_sub_tree(client, callback)?;
		}

		Ok(())
	}
}

impl wl::Object for Surface {
	fn handle(
		&mut self,
		client: &mut Client,
		hw_state: &mut HwState,
		op: u16,
		params: &[u8],
	) -> Result<()> {
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
				let (buffer, x, y): (wl::Id<wl::Buffer>, i32, i32) =
					wlm::decode::from_slice(params)?;

				self.pending.buffer = if !buffer.is_null() {
					Some(buffer)
				} else {
					None
				};

				self.pending.offset = Some((x, y));
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:damage
				let (_x, _y, _w, _h): (i32, i32, i32, i32) = wlm::decode::from_slice(params)?;
			}
			3 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:frame
				let callback: wl::Id<wl::Callback> = wlm::decode::from_slice(params)?;
				client.new_object(callback, wl::Callback::new(callback));

				self.pending.frame_callbacks.push(callback);
			}
			4 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_opaque_region
				let _region: wl::Id<wl::Region> = wlm::decode::from_slice(params)?;
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
				self.commit(client, hw_state)?;
			}
			7 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_buffer_transform
				let _transform: i32 = wlm::decode::from_slice(params)?;
			}
			8 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:set_buffer_scale
				let _scale: i32 = wlm::decode::from_slice(params)?;
			}
			9 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:damage_buffer
				let (_x, _y, _width, _height): (i32, i32, i32, i32) =
					wlm::decode::from_slice(params)?;
			}
			10 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:offset
				let (x, y): (i32, i32) = wlm::decode::from_slice(params)?;
				self.pending.offset = Some((x, y));
			}
			_ => color_eyre::eyre::bail!("unknown op '{op}' in Surface"),
		}

		Ok(())
	}
}
