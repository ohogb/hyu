use color_eyre::eyre::OptionExt as _;

use crate::{Point, Result, gbm};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex {
	pub position: [f32; 2],
	pub uv: [f32; 2],
}

#[derive(Clone)]
pub struct Texture {
	pub image: ash::vk::Image,
	pub image_device_memory: ash::vk::DeviceMemory,
	pub image_view: ash::vk::ImageView,
	pub buffer: ash::vk::Buffer,
	pub buffer_device_memory: ash::vk::DeviceMemory,
	pub buffer_size: usize,
}

impl Vertex {
	pub fn get_binding_description() -> ash::vk::VertexInputBindingDescription {
		ash::vk::VertexInputBindingDescription::default()
			.binding(0)
			.stride(size_of::<Self>() as _)
			.input_rate(ash::vk::VertexInputRate::VERTEX)
	}

	pub fn get_attribute_descriptions() -> [ash::vk::VertexInputAttributeDescription; 2] {
		[
			ash::vk::VertexInputAttributeDescription::default()
				.binding(0)
				.location(0)
				.format(ash::vk::Format::R32G32_SFLOAT)
				.offset(std::mem::offset_of!(Self, position) as _),
			ash::vk::VertexInputAttributeDescription::default()
				.binding(0)
				.location(1)
				.format(ash::vk::Format::R32G32_SFLOAT)
				.offset(std::mem::offset_of!(Self, uv) as _),
		]
	}
}

pub struct Renderer {
	pub entry: ash::Entry,
	pub instance: ash::Instance,
	pub physical_device: ash::vk::PhysicalDevice,
	pub device: ash::Device,
	pub queue: ash::vk::Queue,
	pub queue_family: u32,
	pub command_pool: ash::vk::CommandPool,
	pub pipeline_layout: ash::vk::PipelineLayout,
	pub pipeline: ash::vk::Pipeline,
	pub render_pass: ash::vk::RenderPass,
	pub staging_vertex_buffer: ash::vk::Buffer,
	pub staging_vertex_buffer_device_memory: ash::vk::DeviceMemory,
	pub vertex_buffer: ash::vk::Buffer,
	pub vertex_buffer_device_memory: ash::vk::DeviceMemory,
	pub vertex_buffer_size: usize,
	pub sampler: ash::vk::Sampler,
	pub descriptor_set_layouts: [ash::vk::DescriptorSetLayout; 1],
	pub command_buffer: ash::vk::CommandBuffer,
	pub external_memory_fd_device: ash::khr::external_memory_fd::Device,
	pub push_descriptor: ash::khr::push_descriptor::Device,

	pub vertices: Vec<Vertex>,
	pub textures: Vec<Texture>,
	pub textures_to_delete: Vec<Texture>,

	pub cursor_texture: Texture,
}

pub fn create(card: impl AsRef<std::path::Path>) -> Result<Renderer> {
	let entry = unsafe { ash::Entry::load()? };

	let application_info =
		ash::vk::ApplicationInfo::default().api_version(ash::vk::make_api_version(0, 1, 3, 0));

	let extension_names = [c"VK_KHR_surface".as_ptr(), c"VK_EXT_debug_utils".as_ptr()];
	let layer_names = [c"VK_LAYER_KHRONOS_validation".as_ptr()];

	let instance_create_info = ash::vk::InstanceCreateInfo::default()
		.application_info(&application_info)
		.enabled_extension_names(&extension_names)
		.enabled_layer_names(&layer_names)
		.flags(ash::vk::InstanceCreateFlags::empty());

	let instance = unsafe { entry.create_instance(&instance_create_info, None)? };

	let physical_devices = unsafe { instance.enumerate_physical_devices()? };

	let dev = nix::sys::stat::stat(card.as_ref())?.st_rdev;

	let physical_device = physical_devices
		.iter()
		.cloned()
		.find(|&physical_device| {
			let mut drm_properties = ash::vk::PhysicalDeviceDrmPropertiesEXT::default();
			let mut device_properties =
				ash::vk::PhysicalDeviceProperties2::default().push_next(&mut drm_properties);

			unsafe {
				instance.get_physical_device_properties2(physical_device, &mut device_properties);
			}

			nix::sys::stat::makedev(
				drm_properties.primary_major as _,
				drm_properties.primary_minor as _,
			) == dev
		})
		.ok_or_eyre("didn't find any physical devices")?;

	let queue_family_properties =
		unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

	let queue_family = queue_family_properties
		.iter()
		.enumerate()
		.find(|(_, x)| x.queue_flags.contains(ash::vk::QueueFlags::GRAPHICS))
		.map(|(index, _)| index as u32)
		.ok_or_eyre("failed to find queue with graphics capabilities")?;

	let queue_create_infos = [ash::vk::DeviceQueueCreateInfo::default()
		.queue_priorities(&[1.0])
		.queue_family_index(queue_family)];

	let extension_names = [
		c"VK_EXT_external_memory_dma_buf".as_ptr(),
		c"VK_EXT_image_drm_format_modifier".as_ptr(),
		c"VK_EXT_physical_device_drm".as_ptr(),
		c"VK_EXT_queue_family_foreign".as_ptr(),
		c"VK_KHR_external_memory_fd".as_ptr(),
		c"VK_KHR_push_descriptor".as_ptr(),
	];

	let device_create_info = ash::vk::DeviceCreateInfo::default()
		.queue_create_infos(&queue_create_infos)
		.enabled_extension_names(&extension_names);

	let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };
	let queue = unsafe { device.get_device_queue(queue_family, 0) };

	let command_pool_create_info = ash::vk::CommandPoolCreateInfo::default()
		.flags(ash::vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

	let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };

	let command_buffer_allocation_info = ash::vk::CommandBufferAllocateInfo::default()
		.command_pool(command_pool)
		.level(ash::vk::CommandBufferLevel::PRIMARY)
		.command_buffer_count(1);

	let command_buffers =
		unsafe { device.allocate_command_buffers(&command_buffer_allocation_info)? };

	let command_buffer = command_buffers.into_iter().next().unwrap();

	let vert_shader_code = include_bytes!("vert.spv")
		.chunks(4)
		.map(|x| u32::from_ne_bytes(x.try_into().unwrap()))
		.collect::<Vec<_>>();

	let vert_shader_module_create_info =
		ash::vk::ShaderModuleCreateInfo::default().code(&vert_shader_code);

	let vert_shader_module =
		unsafe { device.create_shader_module(&vert_shader_module_create_info, None)? };

	let frag_shader_code = include_bytes!("frag.spv")
		.chunks(4)
		.map(|x| u32::from_ne_bytes(x.try_into().unwrap()))
		.collect::<Vec<_>>();

	let frag_shader_module_create_info =
		ash::vk::ShaderModuleCreateInfo::default().code(&frag_shader_code);

	let frag_shader_module =
		unsafe { device.create_shader_module(&frag_shader_module_create_info, None)? };

	let pipeline_shader_stage_create_infos = [
		ash::vk::PipelineShaderStageCreateInfo::default()
			.stage(ash::vk::ShaderStageFlags::VERTEX)
			.name(c"main")
			.module(vert_shader_module),
		ash::vk::PipelineShaderStageCreateInfo::default()
			.stage(ash::vk::ShaderStageFlags::FRAGMENT)
			.name(c"main")
			.module(frag_shader_module),
	];

	let vertex_input_binding_descriptions = [Vertex::get_binding_description()];
	let vertex_input_attribute_descriptions = Vertex::get_attribute_descriptions();

	let pipeline_vertex_input_state_create_info =
		ash::vk::PipelineVertexInputStateCreateInfo::default()
			.vertex_binding_descriptions(&vertex_input_binding_descriptions)
			.vertex_attribute_descriptions(&vertex_input_attribute_descriptions);

	let pipeline_input_assembly_state_create_info =
		ash::vk::PipelineInputAssemblyStateCreateInfo::default()
			.topology(ash::vk::PrimitiveTopology::TRIANGLE_LIST);

	let viewports = [ash::vk::Viewport::default()
		.width(2560.0)
		.height(1440.0)
		.min_depth(0.0)
		.max_depth(1.0)];

	let scissors = [
		ash::vk::Rect2D::default().extent(ash::vk::Extent2D::default().width(2560).height(1440))
	];

	let pipeline_viewport_state_create_info = ash::vk::PipelineViewportStateCreateInfo::default()
		.viewports(&viewports)
		.scissors(&scissors);

	let pipeline_rasterization_state_create_info =
		ash::vk::PipelineRasterizationStateCreateInfo::default()
			.polygon_mode(ash::vk::PolygonMode::FILL)
			.cull_mode(ash::vk::CullModeFlags::NONE)
			.front_face(ash::vk::FrontFace::CLOCKWISE)
			.line_width(1.0);

	let pipeline_multisample_state_create_info =
		ash::vk::PipelineMultisampleStateCreateInfo::default()
			.rasterization_samples(ash::vk::SampleCountFlags::TYPE_1);

	let pipeline_color_blend_attachment_states =
		[ash::vk::PipelineColorBlendAttachmentState::default()
			.color_write_mask(ash::vk::ColorComponentFlags::RGBA)
			.blend_enable(true)
			.src_color_blend_factor(ash::vk::BlendFactor::SRC_ALPHA)
			.dst_color_blend_factor(ash::vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
			.color_blend_op(ash::vk::BlendOp::ADD)
			.src_alpha_blend_factor(ash::vk::BlendFactor::ONE)
			.dst_alpha_blend_factor(ash::vk::BlendFactor::ZERO)
			.alpha_blend_op(ash::vk::BlendOp::ADD)];

	let pipeline_color_blend_state_create_info =
		ash::vk::PipelineColorBlendStateCreateInfo::default()
			.attachments(&pipeline_color_blend_attachment_states);

	let descriptor_set_layout_bindings = [ash::vk::DescriptorSetLayoutBinding::default()
		.binding(0)
		.descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
		.descriptor_count(1)
		.stage_flags(ash::vk::ShaderStageFlags::FRAGMENT)];

	let descriptor_set_layout_create_info = ash::vk::DescriptorSetLayoutCreateInfo::default()
		.bindings(&descriptor_set_layout_bindings)
		.flags(ash::vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR);

	let descriptor_set_layouts =
		[
			unsafe {
				device.create_descriptor_set_layout(&descriptor_set_layout_create_info, None)?
			},
		];

	let pipeline_layout_create_info =
		ash::vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_set_layouts);

	let pipeline_layout =
		unsafe { device.create_pipeline_layout(&pipeline_layout_create_info, None)? };

	let attachment_descriptions = [ash::vk::AttachmentDescription::default()
		.format(ash::vk::Format::B8G8R8A8_UNORM)
		.samples(ash::vk::SampleCountFlags::TYPE_1)
		.load_op(ash::vk::AttachmentLoadOp::CLEAR)
		.store_op(ash::vk::AttachmentStoreOp::STORE)
		.stencil_load_op(ash::vk::AttachmentLoadOp::DONT_CARE)
		.stencil_store_op(ash::vk::AttachmentStoreOp::DONT_CARE)
		.initial_layout(ash::vk::ImageLayout::UNDEFINED)
		.final_layout(ash::vk::ImageLayout::GENERAL)];

	let color_attachment_refs = [ash::vk::AttachmentReference::default()
		.layout(ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

	let subpasses = [ash::vk::SubpassDescription::default()
		.pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
		.color_attachments(&color_attachment_refs)];

	let render_pass_create_info = ash::vk::RenderPassCreateInfo::default()
		.attachments(&attachment_descriptions)
		.subpasses(&subpasses);

	let render_pass = unsafe { device.create_render_pass(&render_pass_create_info, None)? };

	let graphics_pipeline_create_infos = [ash::vk::GraphicsPipelineCreateInfo::default()
		.stages(&pipeline_shader_stage_create_infos)
		.vertex_input_state(&pipeline_vertex_input_state_create_info)
		.input_assembly_state(&pipeline_input_assembly_state_create_info)
		.viewport_state(&pipeline_viewport_state_create_info)
		.rasterization_state(&pipeline_rasterization_state_create_info)
		.multisample_state(&pipeline_multisample_state_create_info)
		.color_blend_state(&pipeline_color_blend_state_create_info)
		.layout(pipeline_layout)
		.render_pass(render_pass)];

	let pipeline = unsafe {
		device.create_graphics_pipelines(
			ash::vk::PipelineCache::null(),
			&graphics_pipeline_create_infos,
			None,
		)
	}
	.map_err(|_| color_eyre::eyre::eyre!("failed to create graphics pipeline"))?
	.into_iter()
	.next()
	.unwrap();

	let sampler_create_info = ash::vk::SamplerCreateInfo::default()
		.mag_filter(ash::vk::Filter::LINEAR)
		.min_filter(ash::vk::Filter::LINEAR);

	let sampler = unsafe { device.create_sampler(&sampler_create_info, None)? };

	let staging_vertex_buffer_size = size_of::<Vertex>() * 500;

	let (staging_vertex_buffer, staging_vertex_buffer_device_memory) = Renderer::create_buffer(
		&instance,
		&device,
		physical_device,
		staging_vertex_buffer_size,
		ash::vk::BufferUsageFlags::TRANSFER_SRC,
		ash::vk::MemoryPropertyFlags::HOST_VISIBLE | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
	)?;

	let (vertex_buffer, vertex_buffer_device_memory) = Renderer::create_buffer(
		&instance,
		&device,
		physical_device,
		staging_vertex_buffer_size,
		ash::vk::BufferUsageFlags::TRANSFER_DST | ash::vk::BufferUsageFlags::VERTEX_BUFFER,
		ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
	)?;

	Renderer::copy_buffer_to_buffer(
		&device,
		queue,
		command_pool,
		staging_vertex_buffer,
		vertex_buffer,
		staging_vertex_buffer_size,
	)?;

	let external_memory_fd_device = ash::khr::external_memory_fd::Device::new(&instance, &device);
	let push_descriptor = ash::khr::push_descriptor::Device::new(&instance, &device);

	let (cursor_image, cursor_image_device_memory, cursor_image_view) =
		Renderer::create_image(&device, &instance, physical_device, 2560, 1440, 2560 * 4)?;

	Renderer::clear_image(
		&device,
		queue,
		command_pool,
		cursor_image,
		(0.8, 0.8, 1.0, 1.0),
	)?;

	Ok(Renderer {
		entry,
		instance,
		physical_device,
		device,
		queue,
		queue_family,
		command_pool,
		command_buffer,
		pipeline_layout,
		pipeline,
		render_pass,
		staging_vertex_buffer,
		staging_vertex_buffer_device_memory,
		vertex_buffer,
		vertex_buffer_device_memory,
		vertex_buffer_size: staging_vertex_buffer_size,
		external_memory_fd_device,
		push_descriptor,
		sampler,
		descriptor_set_layouts,
		vertices: vec![],
		textures: vec![],
		textures_to_delete: vec![],
		cursor_texture: Texture {
			image: cursor_image,
			image_device_memory: cursor_image_device_memory,
			image_view: cursor_image_view,
			buffer: ash::vk::Buffer::null(),
			buffer_device_memory: ash::vk::DeviceMemory::null(),
			buffer_size: 0,
		},
	})
}

impl Renderer {
	pub fn create_image_from_gbm(
		&self,
		bo: &gbm::BufferObject,
	) -> Result<(ash::vk::Image, ash::vk::ImageView)> {
		let plane_layouts = [ash::vk::SubresourceLayout::default()
			.offset(0)
			.size(0)
			.row_pitch(bo.get_stride() as _)
			.array_pitch(0)
			.depth_pitch(0)];

		let mut drm_format = ash::vk::ImageDrmFormatModifierExplicitCreateInfoEXT::default()
			.plane_layouts(&plane_layouts)
			.drm_format_modifier(bo.get_modifier());

		let mut external_memory_create_info = ash::vk::ExternalMemoryImageCreateInfo::default()
			.handle_types(ash::vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

		let image_create_info = ash::vk::ImageCreateInfo::default()
			.image_type(ash::vk::ImageType::TYPE_2D)
			.format(ash::vk::Format::B8G8R8A8_UNORM)
			.extent(
				ash::vk::Extent3D::default()
					.width(bo.get_width())
					.height(bo.get_height())
					.depth(1),
			)
			.mip_levels(1)
			.array_layers(1)
			.samples(ash::vk::SampleCountFlags::TYPE_1)
			.usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
			.initial_layout(ash::vk::ImageLayout::UNDEFINED)
			.tiling(ash::vk::ImageTiling::LINEAR)
			.tiling(ash::vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
			.push_next(&mut external_memory_create_info)
			.push_next(&mut drm_format);

		let image = unsafe { self.device.create_image(&image_create_info, None) }?;

		let mut import_memory_fd_into_khr = ash::vk::ImportMemoryFdInfoKHR::default()
			.fd(bo.get_fd())
			.handle_type(ash::vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

		let allocation_info = ash::vk::MemoryAllocateInfo::default()
			.push_next(&mut import_memory_fd_into_khr)
			.allocation_size((bo.get_stride() * bo.get_height()) as _);

		let device_memory = unsafe { self.device.allocate_memory(&allocation_info, None)? };

		unsafe {
			self.device.bind_image_memory(image, device_memory, 0)?;
		}

		let image_view_create_info = ash::vk::ImageViewCreateInfo::default()
			.image(image)
			.view_type(ash::vk::ImageViewType::TYPE_2D)
			.format(ash::vk::Format::B8G8R8A8_UNORM)
			.components(ash::vk::ComponentMapping::default())
			.subresource_range(
				ash::vk::ImageSubresourceRange::default()
					.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
					.level_count(1)
					.layer_count(1),
			);

		let image_view = unsafe {
			self.device
				.create_image_view(&image_view_create_info, None)?
		};

		Ok((image, image_view))
	}

	pub fn create_image_from_dmabuf(
		&self,
		width: u32,
		height: u32,
		format: ash::vk::Format,
		modifier: u64,
		planes: &[crate::wl::Plane],
	) -> Result<(ash::vk::Image, ash::vk::ImageView)> {
		assert!(planes.len() > 0);

		let plane_layouts = planes
			.iter()
			.map(|x| {
				ash::vk::SubresourceLayout::default()
					.offset(x.offset as _)
					.size(0)
					.row_pitch(x.stride as _)
					.array_pitch(0)
					.depth_pitch(0)
			})
			.collect::<Vec<_>>();

		let mut drm_format = ash::vk::ImageDrmFormatModifierExplicitCreateInfoEXT::default()
			.plane_layouts(&plane_layouts)
			.drm_format_modifier(modifier as _);

		let mut external_memory_create_info = ash::vk::ExternalMemoryImageCreateInfo::default()
			.handle_types(ash::vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

		let is_disjoint = (|| -> Result<bool> {
			if planes.len() > 1 {
				let mut other_ino = None;
				for x in planes {
					let ino = nix::sys::stat::fstat(x.fd)?.st_ino;

					if let Some(other_ino) = &other_ino {
						if ino != *other_ino {
							return Ok(true);
						}
					} else {
						other_ino = Some(ino);
					}
				}

				Ok(false)
			} else {
				Ok(false)
			}
		})()?;

		println!("create_image_from_dmabuf() is_disjoint: {is_disjoint}");

		let image_create_info = ash::vk::ImageCreateInfo::default()
			.image_type(ash::vk::ImageType::TYPE_2D)
			.format(format)
			.extent(
				ash::vk::Extent3D::default()
					.width(width as _)
					.height(height as _)
					.depth(1),
			)
			.mip_levels(1)
			.array_layers(1)
			.samples(ash::vk::SampleCountFlags::TYPE_1)
			.usage(ash::vk::ImageUsageFlags::SAMPLED | ash::vk::ImageUsageFlags::TRANSFER_SRC)
			.initial_layout(ash::vk::ImageLayout::UNDEFINED)
			// .tiling(ash::vk::ImageTiling::LINEAR)
			.tiling(ash::vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
			.flags(if is_disjoint {
				ash::vk::ImageCreateFlags::DISJOINT
			} else {
				ash::vk::ImageCreateFlags::default()
			})
			.push_next(&mut external_memory_create_info)
			.push_next(&mut drm_format);

		let image = unsafe { self.device.create_image(&image_create_info, None)? };
		assert!(!ash::vk::Handle::is_null(image));
		println!(" - {image:?}");

		// assert!(!is_disjoint);
		let mut bind_image_memory_infos = [ash::vk::BindImageMemoryInfo::default(); 4];
		let mut bind_image_plane_memory_infos = [ash::vk::BindImagePlaneMemoryInfo::default(); 4];

		let info_iter = std::iter::zip(
			&mut bind_image_memory_infos,
			&mut bind_image_plane_memory_infos,
		);

		let planes_to_iter = if is_disjoint {
			&planes[..]
		} else {
			&planes[..1]
		};

		for (idx, (plane, (memory_info, plane_memory_info))) in
			std::iter::zip(planes_to_iter, info_iter).enumerate()
		{
			let mut memory_fd_properties = ash::vk::MemoryFdPropertiesKHR::default();

			let fd = nix::unistd::dup(plane.fd)?;

			unsafe {
				self.external_memory_fd_device.get_memory_fd_properties(
					ash::vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
					fd,
					&mut memory_fd_properties,
				)?;
			}

			let physical_device_memory_properties = unsafe {
				self.instance
					.get_physical_device_memory_properties(self.physical_device)
			};

			let image_memory_requirements_info =
				ash::vk::ImageMemoryRequirementsInfo2::default().image(image);

			let mut image_memory_requirements = ash::vk::MemoryRequirements2::default();

			unsafe {
				self.device.get_image_memory_requirements2(
					&image_memory_requirements_info,
					&mut image_memory_requirements,
				);
			}
			let memory_type_index = physical_device_memory_properties
				.memory_types
				.iter()
				.enumerate()
				.find(|&(idx, _)| {
					((image_memory_requirements
						.memory_requirements
						.memory_type_bits & memory_fd_properties.memory_type_bits)
						& (1 << idx)) != 0
				})
				.map(|(x, _)| x as u32)
				.ok_or_eyre("physical device doesn't have needed memory property")?;

			let mut import_memory_fd_into_khr = ash::vk::ImportMemoryFdInfoKHR::default()
				.fd(fd)
				.handle_type(ash::vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

			let allocation_info = ash::vk::MemoryAllocateInfo::default()
				.push_next(&mut import_memory_fd_into_khr)
				.allocation_size(image_memory_requirements.memory_requirements.size)
				.memory_type_index(memory_type_index);

			let device_memory = unsafe { self.device.allocate_memory(&allocation_info, None)? };

			*plane_memory_info =
				ash::vk::BindImagePlaneMemoryInfo::default().plane_aspect(match idx {
					0 => ash::vk::ImageAspectFlags::MEMORY_PLANE_0_EXT,
					1 => ash::vk::ImageAspectFlags::MEMORY_PLANE_1_EXT,
					2 => ash::vk::ImageAspectFlags::MEMORY_PLANE_2_EXT,
					3 => ash::vk::ImageAspectFlags::MEMORY_PLANE_3_EXT,
					_ => unreachable!(),
				});

			*memory_info = ash::vk::BindImageMemoryInfo::default()
				.image(image)
				.memory(device_memory);

			if is_disjoint {
				*memory_info = memory_info.push_next(plane_memory_info);
			}
		}

		unsafe {
			self.device
				.bind_image_memory2(&bind_image_memory_infos[..planes_to_iter.len()])?;
		}

		let image_view_create_info = ash::vk::ImageViewCreateInfo::default()
			.image(image)
			.view_type(ash::vk::ImageViewType::TYPE_2D)
			.format(format)
			.components(ash::vk::ComponentMapping::default())
			.components(ash::vk::ComponentMapping::default())
			.subresource_range(
				ash::vk::ImageSubresourceRange::default()
					.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
					.level_count(1)
					.layer_count(1),
			);

		let image_view = unsafe {
			self.device
				.create_image_view(&image_view_create_info, None)?
		};

		Ok((image, image_view))
	}

	pub fn clear_image(
		device: &ash::Device,
		queue: ash::vk::Queue,
		command_pool: ash::vk::CommandPool,
		image: ash::vk::Image,
		color: (f32, f32, f32, f32),
	) -> Result<()> {
		Self::single_time_command(device, queue, command_pool, |command_buffer| {
			let range = ash::vk::ImageSubresourceRange::default()
				.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
				.base_mip_level(0)
				.level_count(1)
				.base_array_layer(0)
				.layer_count(1);

			let image_memory_barrier = ash::vk::ImageMemoryBarrier::default()
				.src_access_mask(ash::vk::AccessFlags::empty())
				.dst_access_mask(ash::vk::AccessFlags::TRANSFER_WRITE)
				.old_layout(ash::vk::ImageLayout::UNDEFINED)
				.new_layout(ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL)
				.src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
				.image(image)
				.subresource_range(range);

			unsafe {
				device.cmd_pipeline_barrier(
					command_buffer,
					ash::vk::PipelineStageFlags::TOP_OF_PIPE,
					ash::vk::PipelineStageFlags::TRANSFER,
					ash::vk::DependencyFlags::empty(),
					&[],
					&[],
					&[image_memory_barrier],
				);
			}

			unsafe {
				device.cmd_clear_color_image(
					command_buffer,
					image,
					ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
					&ash::vk::ClearColorValue {
						float32: color.into(),
					},
					&[range],
				);
			}

			let image_memory_barrier = ash::vk::ImageMemoryBarrier::default()
				.src_access_mask(ash::vk::AccessFlags::TRANSFER_WRITE)
				.dst_access_mask(ash::vk::AccessFlags::SHADER_READ)
				.old_layout(ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL)
				.new_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
				.image(image)
				.subresource_range(range);

			unsafe {
				device.cmd_pipeline_barrier(
					command_buffer,
					ash::vk::PipelineStageFlags::TRANSFER,
					ash::vk::PipelineStageFlags::FRAGMENT_SHADER,
					ash::vk::DependencyFlags::empty(),
					&[],
					&[],
					&[image_memory_barrier],
				);
			}

			Ok(())
		})
	}

	pub fn render(
		&mut self,
		framebuffer_image: ash::vk::Image,
		framebuffer: ash::vk::Framebuffer,
		mut callback: impl FnMut(&mut Renderer) -> Result<()>,
	) -> Result<()> {
		for texture in &self.textures_to_delete {
			unsafe {
				self.device.free_memory(texture.buffer_device_memory, None);
				self.device.destroy_buffer(texture.buffer, None);

				self.device.destroy_image_view(texture.image_view, None);

				self.device.free_memory(texture.image_device_memory, None);
				self.device.destroy_image(texture.image, None);
			}
		}

		self.textures_to_delete.clear();

		let ret;

		let command_buffers = [{
			let command_buffer = self.command_buffer;

			let begin_info = ash::vk::CommandBufferBeginInfo::default()
				.flags(ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

			unsafe {
				self.device
					.begin_command_buffer(command_buffer, &begin_info)?;
			}

			ret = callback(self);

			self.transfer_vertices_to_gpu(command_buffer)?;
			self.setup_texture_barries(command_buffer, framebuffer_image)?;

			let clear_values = [ash::vk::ClearValue {
				color: ash::vk::ClearColorValue {
					float32: [0.2, 0.2, 0.2, 1.0],
				},
			}];

			let render_pass_begin_info = ash::vk::RenderPassBeginInfo::default()
				.render_pass(self.render_pass)
				.framebuffer(framebuffer)
				.render_area(
					ash::vk::Rect2D::default()
						.extent(ash::vk::Extent2D::default().width(2560).height(1440)),
				)
				.clear_values(&clear_values);

			unsafe {
				self.device.cmd_begin_render_pass(
					command_buffer,
					&render_pass_begin_info,
					ash::vk::SubpassContents::INLINE,
				);
			}

			unsafe {
				self.device.cmd_bind_pipeline(
					command_buffer,
					ash::vk::PipelineBindPoint::GRAPHICS,
					self.pipeline,
				);
			}

			self.render_queue(command_buffer)?;

			unsafe {
				self.device.cmd_end_render_pass(command_buffer);
			}

			unsafe {
				self.device.end_command_buffer(command_buffer)?;
			}

			command_buffer
		}];

		let submits = [ash::vk::SubmitInfo::default().command_buffers(&command_buffers)];

		let fence_create_info = ash::vk::FenceCreateInfo::default();

		let fence = unsafe { self.device.create_fence(&fence_create_info, None)? };

		unsafe {
			self.device.queue_submit(self.queue, &submits, fence)?;
		}

		// TODO: set this as IN_FENCE_FD
		unsafe {
			self.device.wait_for_fences(&[fence], true, u64::MAX)?;
		}

		ret
	}

	pub fn create_buffer(
		instance: &ash::Instance,
		device: &ash::Device,
		physical_device: ash::vk::PhysicalDevice,
		size: usize,
		usage: ash::vk::BufferUsageFlags,
		properties: ash::vk::MemoryPropertyFlags,
	) -> Result<(ash::vk::Buffer, ash::vk::DeviceMemory)> {
		let buffer_create_info = ash::vk::BufferCreateInfo::default()
			.size(size as _)
			.usage(usage)
			.sharing_mode(ash::vk::SharingMode::EXCLUSIVE);

		let buffer = unsafe { device.create_buffer(&buffer_create_info, None)? };

		let buffer_memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

		let physical_device_memory_properties =
			unsafe { instance.get_physical_device_memory_properties(physical_device) };

		let buffer_memory_allocation_info = ash::vk::MemoryAllocateInfo::default()
			.allocation_size(buffer_memory_requirements.size)
			.memory_type_index(
				physical_device_memory_properties
					.memory_types
					.iter()
					.enumerate()
					.find(|&(idx, x)| {
						(buffer_memory_requirements.memory_type_bits & (1 << idx)) != 0
							&& x.property_flags.contains(properties)
					})
					.map(|(x, _)| x as u32)
					.ok_or_eyre("physical device doesn't have needed memory property")?,
			);

		let buffer_device_memory =
			unsafe { device.allocate_memory(&buffer_memory_allocation_info, None)? };

		unsafe {
			device.bind_buffer_memory(buffer, buffer_device_memory, 0)?;
		}

		Ok((buffer, buffer_device_memory))
	}

	pub fn create_image(
		device: &ash::Device,
		instance: &ash::Instance,
		physical_device: ash::vk::PhysicalDevice,
		width: usize,
		height: usize,
		stride: usize,
	) -> Result<(ash::vk::Image, ash::vk::DeviceMemory, ash::vk::ImageView)> {
		let image_create_info = ash::vk::ImageCreateInfo::default()
			.image_type(ash::vk::ImageType::TYPE_2D)
			.format(ash::vk::Format::B8G8R8A8_UNORM)
			.extent(
				ash::vk::Extent3D::default()
					.width(width as _)
					.height(height as _)
					.depth(1),
			)
			.mip_levels(1)
			.array_layers(1)
			.samples(ash::vk::SampleCountFlags::TYPE_1)
			.usage(ash::vk::ImageUsageFlags::TRANSFER_DST | ash::vk::ImageUsageFlags::SAMPLED)
			.initial_layout(ash::vk::ImageLayout::UNDEFINED)
			.tiling(ash::vk::ImageTiling::LINEAR)
			.tiling(ash::vk::ImageTiling::OPTIMAL);

		let image = unsafe { device.create_image(&image_create_info, None) }?;

		let image_memory_requirements = unsafe { device.get_image_memory_requirements(image) };

		let physical_device_memory_properties =
			unsafe { instance.get_physical_device_memory_properties(physical_device) };

		let allocation_info = ash::vk::MemoryAllocateInfo::default()
			.allocation_size(image_memory_requirements.size)
			.memory_type_index(
				physical_device_memory_properties
					.memory_types
					.iter()
					.enumerate()
					.find(|&(idx, x)| {
						(image_memory_requirements.memory_type_bits & (1 << idx)) != 0
							&& x.property_flags.contains(
								ash::vk::MemoryPropertyFlags::HOST_VISIBLE
									| ash::vk::MemoryPropertyFlags::HOST_COHERENT,
							)
					})
					.map(|(x, _)| x as u32)
					.ok_or_eyre("physical device doesn't have needed memory property")?,
			);

		let device_memory = unsafe { device.allocate_memory(&allocation_info, None)? };

		unsafe {
			device.bind_image_memory(image, device_memory, 0)?;
		}

		let image_view_create_info = ash::vk::ImageViewCreateInfo::default()
			.image(image)
			.view_type(ash::vk::ImageViewType::TYPE_2D)
			.format(ash::vk::Format::B8G8R8A8_UNORM)
			.components(ash::vk::ComponentMapping::default())
			.subresource_range(
				ash::vk::ImageSubresourceRange::default()
					.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
					.level_count(1)
					.layer_count(1),
			);

		let image_view = unsafe { device.create_image_view(&image_view_create_info, None)? };

		Ok((image, device_memory, image_view))
	}

	pub fn copy_to_buffer(
		device: &ash::Device,
		buffer_device_memory: ash::vk::DeviceMemory,
		buffer_size: usize,
		data: &[u8],
	) -> Result<()> {
		assert!(buffer_size >= data.len(), "{} {}", buffer_size, data.len());

		unsafe {
			let ptr = device.map_memory(
				buffer_device_memory,
				0,
				buffer_size as _,
				ash::vk::MemoryMapFlags::default(),
			)?;

			std::ptr::copy(data.as_ptr(), ptr as *mut u8, data.len());

			device.unmap_memory(buffer_device_memory);
		}

		Ok(())
	}

	pub fn copy_buffer_to_image(
		device: &ash::Device,
		queue: ash::vk::Queue,
		command_pool: ash::vk::CommandPool,
		buffer: ash::vk::Buffer,
		image: ash::vk::Image,
		width: u32,
		height: u32,
	) -> Result<()> {
		Self::single_time_command(
			device,
			queue,
			command_pool,
			|command_buffer| -> Result<()> {
				let regions = [ash::vk::BufferImageCopy::default()
					.buffer_offset(0)
					.buffer_row_length(0)
					.buffer_image_height(0)
					.image_subresource(
						ash::vk::ImageSubresourceLayers::default()
							.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
							.mip_level(0)
							.base_array_layer(0)
							.layer_count(1),
					)
					.image_offset(ash::vk::Offset3D::default())
					.image_extent(
						ash::vk::Extent3D::default()
							.width(width)
							.height(height)
							.depth(1),
					)];

				unsafe {
					device.cmd_copy_buffer_to_image(
						command_buffer,
						buffer,
						image,
						ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
						&regions,
					);
				}

				Ok(())
			},
		)
	}

	pub fn copy_buffer_to_buffer(
		device: &ash::Device,
		queue: ash::vk::Queue,
		command_pool: ash::vk::CommandPool,
		from: ash::vk::Buffer,
		to: ash::vk::Buffer,
		size: usize,
	) -> Result<()> {
		Self::single_time_command(
			device,
			queue,
			command_pool,
			|command_buffer| -> Result<()> {
				let regions = [ash::vk::BufferCopy::default().size(size as _)];

				unsafe {
					device.cmd_copy_buffer(command_buffer, from, to, &regions);
				}

				Ok(())
			},
		)
	}

	pub fn single_time_command(
		device: &ash::Device,
		queue: ash::vk::Queue,
		command_pool: ash::vk::CommandPool,
		mut callback: impl FnMut(ash::vk::CommandBuffer) -> Result<()>,
	) -> Result<()> {
		let ret;
		let command_buffers = [{
			let command_buffer_allocation_info = ash::vk::CommandBufferAllocateInfo::default()
				.command_pool(command_pool)
				.level(ash::vk::CommandBufferLevel::PRIMARY)
				.command_buffer_count(1);

			let command_buffers =
				unsafe { device.allocate_command_buffers(&command_buffer_allocation_info)? };

			let command_buffer = command_buffers.into_iter().next().unwrap();

			let begin_info = ash::vk::CommandBufferBeginInfo::default()
				.flags(ash::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

			unsafe {
				device.begin_command_buffer(command_buffer, &begin_info)?;
			}

			ret = callback(command_buffer);

			unsafe {
				device.end_command_buffer(command_buffer)?;
			}

			command_buffer
		}];

		let submits = [ash::vk::SubmitInfo::default().command_buffers(&command_buffers)];

		unsafe {
			device.queue_submit(queue, &submits, ash::vk::Fence::null())?;
		}

		unsafe {
			device.queue_wait_idle(queue)?;
		}

		unsafe {
			device.free_command_buffers(command_pool, &command_buffers);
		}

		ret
	}

	pub fn transition_image_layout(
		device: &ash::Device,
		queue: ash::vk::Queue,
		command_pool: ash::vk::CommandPool,
		image: ash::vk::Image,
		old_layout: ash::vk::ImageLayout,
		new_layout: ash::vk::ImageLayout,
	) -> Result<()> {
		Self::single_time_command(device, queue, command_pool, |command_buffer| {
			let range = ash::vk::ImageSubresourceRange::default()
				.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
				.base_mip_level(0)
				.level_count(1)
				.base_array_layer(0)
				.layer_count(1);

			let image_memory_barrier = ash::vk::ImageMemoryBarrier::default()
				.src_access_mask(ash::vk::AccessFlags::empty())
				.dst_access_mask(ash::vk::AccessFlags::TRANSFER_WRITE)
				.old_layout(old_layout)
				.new_layout(new_layout)
				.src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
				.image(image)
				.subresource_range(range);

			unsafe {
				device.cmd_pipeline_barrier(
					command_buffer,
					ash::vk::PipelineStageFlags::TOP_OF_PIPE,
					ash::vk::PipelineStageFlags::TRANSFER,
					ash::vk::DependencyFlags::empty(),
					&[],
					&[],
					&[image_memory_barrier],
				);
			}

			Ok(())
		})
	}

	pub fn record_quad(&mut self, position: Point, size: Point, texture: &Texture) -> Result<()> {
		let pixels_to_float = |input: [i32; 2]| -> [f32; 2] {
			[
				input[0] as f32 / 2560 as f32 * 2.0 - 1.0,
				(input[1] as f32 / 1440 as f32 * 2.0 - 1.0),
			]
		};

		let Point(x, y) = position;
		let Point(width, height) = size;

		self.vertices.extend([
			Vertex {
				position: pixels_to_float([x, y]),
				uv: [0.0, 0.0],
			},
			Vertex {
				position: pixels_to_float([x + width, y]),
				uv: [1.0, 0.0],
			},
			Vertex {
				position: pixels_to_float([x, y + height]),
				uv: [0.0, 1.0],
			},
			Vertex {
				position: pixels_to_float([x, y + height]),
				uv: [0.0, 1.0],
			},
			Vertex {
				position: pixels_to_float([x + width, y + height]),
				uv: [1.0, 1.0],
			},
			Vertex {
				position: pixels_to_float([x + width, y]),
				uv: [1.0, 0.0],
			},
		]);

		self.textures.push(texture.clone());
		Ok(())
	}

	pub fn transfer_vertices_to_gpu(
		&mut self,
		command_buffer: ash::vk::CommandBuffer,
	) -> Result<()> {
		assert!(self.vertices.len() % 6 == 0);
		assert!(self.vertices.len() / 6 == self.textures.len());

		Renderer::copy_to_buffer(
			&self.device,
			self.staging_vertex_buffer_device_memory,
			self.vertex_buffer_size,
			bytemuck::cast_slice(&self.vertices),
		)?;

		self.vertices.clear();

		let regions = [ash::vk::BufferCopy::default().size(self.vertex_buffer_size as _)];

		unsafe {
			self.device.cmd_copy_buffer(
				command_buffer,
				self.staging_vertex_buffer,
				self.vertex_buffer,
				&regions,
			);
		}

		Ok(())
	}

	pub fn setup_texture_barries(
		&mut self,
		command_buffer: ash::vk::CommandBuffer,
		framebuffer_image: ash::vk::Image,
	) -> Result<()> {
		let acquire_barries = self
			.textures
			.iter()
			.map(|texture| {
				let range = ash::vk::ImageSubresourceRange::default()
					.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1);

				ash::vk::ImageMemoryBarrier::default()
					.src_access_mask(ash::vk::AccessFlags::empty())
					.dst_access_mask(ash::vk::AccessFlags::SHADER_READ)
					.old_layout(ash::vk::ImageLayout::GENERAL)
					.new_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
					.src_queue_family_index(ash::vk::QUEUE_FAMILY_FOREIGN_EXT)
					.dst_queue_family_index(self.queue_family)
					.image(texture.image)
					.subresource_range(range)
			})
			.chain(std::iter::once_with(|| {
				let range = ash::vk::ImageSubresourceRange::default()
					.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1);

				ash::vk::ImageMemoryBarrier::default()
					.src_access_mask(ash::vk::AccessFlags::empty())
					.dst_access_mask(
						ash::vk::AccessFlags::COLOR_ATTACHMENT_READ
							| ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
					)
					.old_layout(ash::vk::ImageLayout::GENERAL)
					.new_layout(ash::vk::ImageLayout::GENERAL)
					.src_queue_family_index(ash::vk::QUEUE_FAMILY_FOREIGN_EXT)
					.dst_queue_family_index(self.queue_family)
					.image(framebuffer_image)
					.subresource_range(range)
			}))
			.collect::<Vec<_>>();

		let release_barries = self
			.textures
			.iter()
			.map(|texture| {
				let range = ash::vk::ImageSubresourceRange::default()
					.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1);

				ash::vk::ImageMemoryBarrier::default()
					.src_access_mask(ash::vk::AccessFlags::SHADER_READ)
					.dst_access_mask(ash::vk::AccessFlags::empty())
					.old_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
					.new_layout(ash::vk::ImageLayout::GENERAL)
					.src_queue_family_index(self.queue_family)
					.dst_queue_family_index(ash::vk::QUEUE_FAMILY_FOREIGN_EXT)
					.image(texture.image)
					.subresource_range(range)
			})
			.chain(std::iter::once_with(|| {
				let range = ash::vk::ImageSubresourceRange::default()
					.aspect_mask(ash::vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1);

				ash::vk::ImageMemoryBarrier::default()
					.src_access_mask(
						ash::vk::AccessFlags::COLOR_ATTACHMENT_READ
							| ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
					)
					.dst_access_mask(ash::vk::AccessFlags::empty())
					.old_layout(ash::vk::ImageLayout::GENERAL)
					.new_layout(ash::vk::ImageLayout::GENERAL)
					.src_queue_family_index(self.queue_family)
					.dst_queue_family_index(ash::vk::QUEUE_FAMILY_FOREIGN_EXT)
					.image(framebuffer_image)
					.subresource_range(range)
			}))
			.collect::<Vec<_>>();

		unsafe {
			self.device.cmd_pipeline_barrier(
				command_buffer,
				ash::vk::PipelineStageFlags::TOP_OF_PIPE,
				ash::vk::PipelineStageFlags::FRAGMENT_SHADER
					| ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
				ash::vk::DependencyFlags::empty(),
				&[],
				&[],
				&acquire_barries,
			);

			self.device.cmd_pipeline_barrier(
				command_buffer,
				ash::vk::PipelineStageFlags::ALL_GRAPHICS,
				ash::vk::PipelineStageFlags::BOTTOM_OF_PIPE,
				ash::vk::DependencyFlags::empty(),
				&[],
				&[],
				&release_barries,
			);
		}

		Ok(())
	}

	pub fn render_queue(&mut self, command_buffer: ash::vk::CommandBuffer) -> Result<()> {
		assert!(self.vertices.is_empty());

		unsafe {
			let buffers = [self.vertex_buffer];
			let offsets = [0];

			self.device
				.cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &offsets);
		}

		for (idx, texture) in self.textures.iter().enumerate() {
			let descriptor_image_infos = [ash::vk::DescriptorImageInfo::default()
				.image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.image_view(texture.image_view)
				.sampler(self.sampler)];

			let descriptor_writes = [ash::vk::WriteDescriptorSet::default()
				.dst_set(ash::vk::DescriptorSet::null())
				.dst_binding(0)
				.descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
				.descriptor_count(1)
				.image_info(&descriptor_image_infos)];

			unsafe {
				self.push_descriptor.cmd_push_descriptor_set(
					command_buffer,
					ash::vk::PipelineBindPoint::GRAPHICS,
					self.pipeline_layout,
					0,
					&descriptor_writes,
				);
			}

			unsafe {
				self.device
					.cmd_draw(command_buffer, 6, 1, (idx * 6) as _, 0);
			}
		}

		self.textures.clear();
		Ok(())
	}
}
