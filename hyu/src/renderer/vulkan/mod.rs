use color_eyre::eyre::OptionExt as _;

use crate::{Result, gbm};

pub struct Renderer {
	pub entry: ash::Entry,
	pub instance: ash::Instance,
	pub physical_device: ash::vk::PhysicalDevice,
	pub device: ash::Device,
	pub queue: ash::vk::Queue,
	pub command_pool: ash::vk::CommandPool,
	pub pipeline: ash::vk::Pipeline,
	pub render_pass: ash::vk::RenderPass,
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
		c"VK_EXT_physical_device_drm".as_ptr(),
		c"VK_EXT_image_drm_format_modifier".as_ptr(),
		c"VK_KHR_external_memory_fd".as_ptr(),
		c"VK_EXT_external_memory_dma_buf".as_ptr(),
	];

	let device_create_info = ash::vk::DeviceCreateInfo::default()
		.queue_create_infos(&queue_create_infos)
		.enabled_extension_names(&extension_names);

	let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };
	let queue = unsafe { device.get_device_queue(queue_family, 0) };

	let command_pool_create_info = ash::vk::CommandPoolCreateInfo::default();
	let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };

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

	let pipeline_vertex_input_state_create_info =
		ash::vk::PipelineVertexInputStateCreateInfo::default();

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

	let pipeline_layout_create_info = ash::vk::PipelineLayoutCreateInfo::default();

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

	Ok(Renderer {
		entry,
		instance,
		physical_device,
		device,
		queue,
		command_pool,
		pipeline,
		render_pass,
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

	pub fn clear_image(&self, image: ash::vk::Image, color: (f32, f32, f32, f32)) -> Result<()> {
		let command_buffers = [{
			let command_buffer_allocation_info = ash::vk::CommandBufferAllocateInfo::default()
				.command_pool(self.command_pool)
				.level(ash::vk::CommandBufferLevel::PRIMARY)
				.command_buffer_count(1);

			let command_buffers = unsafe {
				self.device
					.allocate_command_buffers(&command_buffer_allocation_info)?
			};

			let command_buffer = command_buffers.into_iter().next().unwrap();

			let begin_info = ash::vk::CommandBufferBeginInfo::default()
				.flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

			unsafe {
				self.device
					.begin_command_buffer(command_buffer, &begin_info)?;
			}

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
				self.device.cmd_pipeline_barrier(
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
				self.device.cmd_clear_color_image(
					command_buffer,
					image,
					ash::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
					&ash::vk::ClearColorValue {
						float32: color.into(),
					},
					&[range],
				);
			}

			unsafe {
				self.device.end_command_buffer(command_buffer)?;
			}

			command_buffer
		}];

		let submits = [ash::vk::SubmitInfo::default().command_buffers(&command_buffers)];

		unsafe {
			self.device
				.queue_submit(self.queue, &submits, ash::vk::Fence::null())?;
		}

		Ok(())
	}

	pub fn render(&self, framebuffer: ash::vk::Framebuffer) -> Result<()> {
		let command_buffers = [{
			let command_buffer_allocation_info = ash::vk::CommandBufferAllocateInfo::default()
				.command_pool(self.command_pool)
				.level(ash::vk::CommandBufferLevel::PRIMARY)
				.command_buffer_count(1);

			let command_buffers = unsafe {
				self.device
					.allocate_command_buffers(&command_buffer_allocation_info)?
			};

			let command_buffer = command_buffers.into_iter().next().unwrap();

			let begin_info = ash::vk::CommandBufferBeginInfo::default()
				.flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

			unsafe {
				self.device
					.begin_command_buffer(command_buffer, &begin_info)?;
			}

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

			unsafe {
				self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
			}

			unsafe {
				self.device.cmd_end_render_pass(command_buffer);
			}

			unsafe {
				self.device.end_command_buffer(command_buffer)?;
			}

			command_buffer
		}];

		let submits = [ash::vk::SubmitInfo::default().command_buffers(&command_buffers)];

		unsafe {
			self.device
				.queue_submit(self.queue, &submits, ash::vk::Fence::null())?;
		}

		Ok(())
	}
}
