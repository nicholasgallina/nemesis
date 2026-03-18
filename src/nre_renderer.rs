use crate::nre_device::NreDevice;
use crate::nre_swap_chain::NreSwapChain;
use ash::vk;

pub struct NreRenderer {
    swap_chain: NreSwapChain,
    command_buffers: Vec<vk::CommandBuffer>,
    current_image_index: u32,
    current_frame_index: usize,
    is_frame_started: bool,
}

impl NreRenderer {
    pub fn new(device: &NreDevice, extent: vk::Extent2D) -> Self {
        let swap_chain = NreSwapChain::new(device, extent);
        let command_buffers = Self::create_command_buffers(device);
        Self {
            swap_chain,
            command_buffers,
            current_image_index: 0,
            current_frame_index: 0,
            is_frame_started: false,
        }
    }

    fn create_command_buffers(device: &NreDevice) -> Vec<vk::CommandBuffer> {
        let alloc_info = vk::CommandBufferAllocateInfo {
            command_pool: device.command_pool(),
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 2,
            ..Default::default()
        };
        unsafe {
            device
                .device()
                .allocate_command_buffers(&alloc_info)
                .unwrap()
        }
    }

    fn create_command_pool(device: &ash::Device, queue_family_index: u32) -> vk::CommandPool {
        let pool_info = vk::CommandPoolCreateInfo {
            queue_family_index,
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
                | vk::CommandPoolCreateFlags::TRANSIENT,
            ..Default::default()
        };
        unsafe { device.create_command_pool(&pool_info, None).unwrap() }
    }
}
