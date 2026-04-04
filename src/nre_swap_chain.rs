use std::mem::swap;

use crate::nre_device::NreDevice;
use ash::vk;

pub struct NreSwapChain {
    pub swapchain_loader: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    images_format: vk::Format,
    pub extent: vk::Extent2D,
    image_views: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
}

impl NreSwapChain {
    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }

    pub fn new(device: &NreDevice, extent: vk::Extent2D) -> Self {
        let support = SwapChainSupportDetails::query(device);
        let surface_format = Self::choose_surface_format(&support.formats);
        let present_mode = Self::choose_present_mode(&support.present_modes);
        let swap_extent = Self::choose_extent(&support.capabilities, extent);

        let image_count = (support.capabilities.min_image_count + 1).min(
            if support.capabilities.max_image_count > 0 {
                support.capabilities.max_image_count
            } else {
                u32::MAX
            },
        );

        let create_info = vk::SwapchainCreateInfoKHR {
            surface: device.surface(),
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: swap_extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };

        let swapchain_loader = ash::khr::swapchain::Device::new(device.instance(), device.device());
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .unwrap()
        };

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };

        let image_views = Self::create_image_views(&images, surface_format.format, device.device());
        let render_pass = Self::create_render_pass(device.device(), surface_format.format);
        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
            Self::create_sync_objects(device.device(), images.len());

        let (depth_image, depth_image_memory, depth_image_view) =
            Self::create_depth_resources(device, swap_extent);
        let framebuffers = Self::create_framebuffers(
            device.device(),
            &image_views,
            render_pass,
            swap_extent,
            depth_image_view,
        );

        Self {
            swapchain_loader,
            swapchain,
            images,
            images_format: surface_format.format,
            extent: swap_extent,
            image_views,
            render_pass,
            framebuffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            depth_image,
            depth_image_memory,
            depth_image_view,
        }
    }

    fn create_render_pass(device: &ash::Device, format: vk::Format) -> vk::RenderPass {
        let color_attachment = vk::AttachmentDescription {
            format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };

        let depth_attachment = vk::AttachmentDescription {
            format: vk::Format::D32_SFLOAT,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: &depth_attachment_ref,
            ..Default::default()
        };

        let dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let attachments = [color_attachment, depth_attachment];
        let render_pass_info = vk::RenderPassCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &dependency,
            ..Default::default()
        };

        unsafe { device.create_render_pass(&render_pass_info, None).unwrap() }
    }

    fn create_sync_objects(
        device: &ash::Device,
        count: usize,
    ) -> (Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>) {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo {
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let mut image_available = vec![];
        let mut render_finished = vec![];
        let mut fences = vec![];

        for _ in 0..count {
            unsafe {
                image_available.push(device.create_semaphore(&semaphore_info, None).unwrap());
                render_finished.push(device.create_semaphore(&semaphore_info, None).unwrap());
                fences.push(device.create_fence(&fence_info, None).unwrap());
            }
        }

        (image_available, render_finished, fences)
    }

    fn create_image_views(
        images: &[vk::Image],
        format: vk::Format,
        device: &ash::Device,
    ) -> Vec<vk::ImageView> {
        images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo {
                    image,
                    view_type: vk::ImageViewType::TYPE_2D,
                    format,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };
                unsafe { device.create_image_view(&create_info, None).unwrap() }
            })
            .collect()
    }

    fn create_framebuffers(
        device: &ash::Device,
        image_views: &[vk::ImageView],
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        depth_image_view: vk::ImageView,
    ) -> Vec<vk::Framebuffer> {
        image_views
            .iter()
            .map(|&view| {
                let attachments = [view, depth_image_view];
                let framebuffer_info = vk::FramebufferCreateInfo {
                    render_pass,
                    attachment_count: 2,
                    p_attachments: attachments.as_ptr(),
                    width: extent.width,
                    height: extent.height,
                    layers: 1,
                    ..Default::default()
                };
                unsafe { device.create_framebuffer(&framebuffer_info, None).unwrap() }
            })
            .collect()
    }

    fn choose_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .copied()
            .unwrap_or(formats[0])
    }

    fn choose_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            return vk::PresentModeKHR::MAILBOX;
        }
        vk::PresentModeKHR::FIFO
    }

    fn choose_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        window_extent: vk::Extent2D,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        }
        vk::Extent2D {
            width: window_extent.width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: window_extent.height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }

    pub fn create_depth_resources(
        device: &NreDevice,
        extent: vk::Extent2D,
    ) -> (vk::Image, vk::DeviceMemory, vk::ImageView) {
        let image_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::D32_SFLOAT,
            extent: vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let image = unsafe { device.device().create_image(&image_info, None).unwrap() };

        let mem_requirements = unsafe { device.device().get_image_memory_requirements(image) };
        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: mem_requirements.size,
            memory_type_index: crate::nre_model::NreModel::find_memory_type(
                device,
                mem_requirements.memory_type_bits,
            ),
            ..Default::default()
        };
        let memory = unsafe { device.device().allocate_memory(&alloc_info, None).unwrap() };
        unsafe { device.device().bind_image_memory(image, memory, 0).unwrap() };

        let view_info = vk::ImageViewCreateInfo {
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: vk::Format::D32_SFLOAT,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        let image_view = unsafe { device.device().create_image_view(&view_info, None).unwrap() };

        (image, memory, image_view)
    }

    //
}

struct SwapChainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapChainSupportDetails {
    fn query(device: &NreDevice) -> Self {
        let capabilities = unsafe {
            device
                .surface_loader()
                .get_physical_device_surface_capabilities(
                    device.physical_device(),
                    device.surface(),
                )
                .unwrap()
        };

        let formats = unsafe {
            device
                .surface_loader()
                .get_physical_device_surface_formats(device.physical_device(), device.surface())
                .unwrap()
        };

        let present_modes = unsafe {
            device
                .surface_loader()
                .get_physical_device_surface_present_modes(
                    device.physical_device(),
                    device.surface(),
                )
                .unwrap()
        };

        Self {
            capabilities,
            formats,
            present_modes,
        }
    }
}
