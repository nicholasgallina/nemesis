use crate::nre_device::NreDevice;
use ash::vk;

pub struct NreSwapChain {
    swapchain_loader: ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    images_format: vk::Format,
    extent: vk::Extent2D,
    image_views: Vec<vk::ImageView>,
}

impl NreSwapChain {
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

        Self {
            swapchain_loader,
            swapchain,
            images,
            images_format: surface_format.format,
            extent: swap_extent,
            image_views,
        }
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
