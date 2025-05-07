use std::rc::Rc;

use crate::{
	Client, Connection, Point, Result,
	renderer::{self},
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
		if let x @ Some(_) = std::mem::take(&mut self.buffer) {
			other.buffer = x;
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

#[derive(Default)]
pub enum SurfaceRenderTexture {
	#[default]
	None,
	UnattachedShmCopy((Point, renderer::vulkan::Texture)),
	AttachedDmabuf(AttachedBuffer),
}

pub struct AttachedBuffer {
	pub wl_buffer_id: wl::Id<wl::Buffer>,
	ref_count: std::rc::Rc<std::cell::RefCell<usize>>,
}

impl AttachedBuffer {
	pub fn new(wl_buffer_id: wl::Id<wl::Buffer>) -> Self {
		Self {
			wl_buffer_id,
			ref_count: std::rc::Rc::new(std::cell::RefCell::new(1)),
		}
	}

	pub fn release(&self, client: &mut Client) -> Result<()> {
		let mut ref_count = self.ref_count.borrow_mut();
		assert!(*ref_count > 0);
		*ref_count -= 1;

		if *ref_count == 0 {
			let Ok(wl_buffer) = client.get_object(self.wl_buffer_id) else {
				return Ok(());
			};
			wl_buffer.release()?;
		}

		Ok(())
	}

	pub fn clone(&self) -> Self {
		*self.ref_count.borrow_mut() += 1;

		Self {
			wl_buffer_id: self.wl_buffer_id,
			ref_count: self.ref_count.clone(),
		}
	}
}

impl Drop for AttachedBuffer {
	fn drop(&mut self) {
		if std::rc::Rc::strong_count(&self.ref_count) == 1 {
			assert!(*self.ref_count.borrow() == 0, "{}", *self.wl_buffer_id);
		}
	}
}

pub struct Surface {
	pub object_id: wl::Id<Self>,
	conn: Rc<Connection>,
	pub children: Vec<wl::Id<wl::SubSurface>>,
	pub role: Option<SurfaceRole>,
	pub pending: SurfaceState,
	pub current: SurfaceState,
	pub render_texture: SurfaceRenderTexture,
	pub currently_rendered_buffer: Option<AttachedBuffer>,
	pub currently_displaying_buffer: Option<AttachedBuffer>,
}

impl Surface {
	pub fn new(object_id: wl::Id<Self>, conn: Rc<Connection>) -> Self {
		Self {
			object_id,
			conn,
			children: Vec::new(),
			role: None,
			pending: Default::default(),
			current: Default::default(),
			render_texture: SurfaceRenderTexture::None,
			currently_rendered_buffer: None,
			currently_displaying_buffer: None,
		}
	}

	pub fn push(&mut self, child: wl::Id<wl::SubSurface>) {
		self.children.push(child);
	}

	pub fn get_front_buffers(&self, client: &Client) -> Result<Vec<(Point, wl::Id<wl::Surface>)>> {
		if let SurfaceRenderTexture::None = &self.render_texture {
			return Ok(Vec::new());
		}

		let mut ret = Vec::new();
		ret.push((Point(0, 0), self.object_id));

		for i in &self.children {
			let sub_surface = client.get_object(*i)?;
			let surface = client.get_object(sub_surface.surface)?;

			let position = sub_surface.position;

			ret.extend(
				surface
					.get_front_buffers(client)?
					.into_iter()
					.map(|x| (x.0 + position, x.1)),
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

			presentation_feedback.sync_output(output)?;
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

	pub fn vk_do_textures(
		&mut self,
		client: &mut Client,
		vk: &mut renderer::vulkan::Renderer,
	) -> Result<()> {
		let Some(wl_buffer_id) = std::mem::take(&mut self.current.buffer) else {
			return Ok(());
		};

		if wl_buffer_id.is_null() {
			match std::mem::take(&mut self.render_texture) {
				SurfaceRenderTexture::None => {}
				SurfaceRenderTexture::UnattachedShmCopy((_, texture)) => {
					vk.textures_to_delete.push(texture);
				}
				SurfaceRenderTexture::AttachedDmabuf(attached_buffer) => {
					attached_buffer.release(client)?;
				}
			}

			return Ok(());
		}

		let wl_buffer = client.get_object(wl_buffer_id)?;

		match &wl_buffer.backing_storage {
			wl::BufferBackingStorage::Shm(shm_backing_storage) => {
				let mut render_texture = match std::mem::take(&mut self.render_texture) {
					SurfaceRenderTexture::None => None,
					SurfaceRenderTexture::UnattachedShmCopy(x) => Some(x),
					SurfaceRenderTexture::AttachedDmabuf { .. } => todo!(),
				};

				if let &Some((size, _)) = &render_texture {
					if size != shm_backing_storage.size {
						render_texture = None;
					}
				}

				shm_backing_storage.copy_into_texture(vk, &mut render_texture)?;

				let Some(render_texture) = render_texture else {
					panic!();
				};

				self.render_texture = SurfaceRenderTexture::UnattachedShmCopy(render_texture);

				wl_buffer.release()?;
			}
			wl::BufferBackingStorage::Dmabuf(_) => {
				match std::mem::take(&mut self.render_texture) {
					SurfaceRenderTexture::None => {}
					SurfaceRenderTexture::UnattachedShmCopy(_) => todo!(),
					SurfaceRenderTexture::AttachedDmabuf(attached_buffer) => {
						attached_buffer.release(client)?;
					}
				};

				self.render_texture =
					SurfaceRenderTexture::AttachedDmabuf(AttachedBuffer::new(wl_buffer_id));
			}
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
				wlr_layer_surface.configure(wl_display.new_serial(), 0, 0)?;

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

		self.vk_do_textures(client, &mut hw_state.drm.vulkan)?;

		self.depth_first_sub_tree(client, &mut |client, _, surface| {
			let Some(SurfaceRole::SubSurface { mode, .. }) = &mut surface.role else {
				panic!();
			};

			// TODO: check if parent should override mode
			if let SubSurfaceMode::Sync { state_to_apply } = mode {
				state_to_apply.apply_to(&mut surface.current);
				surface.vk_do_textures(client, &mut hw_state.drm.vulkan)?;
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
				if let SurfaceRenderTexture::AttachedDmabuf(attached_buffer) = &self.render_texture
				{
					attached_buffer.release(client)?;
				}

				if let Some(attached_buffer) = &self.currently_rendered_buffer {
					attached_buffer.release(client)?;
				}

				if let Some(attached_buffer) = &self.currently_displaying_buffer {
					attached_buffer.release(client)?;
				}

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

				self.pending.buffer = Some(buffer);
				self.pending.offset = Some((x, y));
			}
			2 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:damage
				let (_x, _y, _w, _h): (i32, i32, i32, i32) = wlm::decode::from_slice(params)?;
			}
			3 => {
				// https://wayland.app/protocols/wayland#wl_surface:request:frame
				let callback: wl::Id<wl::Callback> = wlm::decode::from_slice(params)?;
				client.new_object(callback, wl::Callback::new(callback, self.conn.clone()));

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
					wl::Region::new(wl::Id::null(), self.conn.clone())
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
