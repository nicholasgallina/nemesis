use crate::nre_device::NreDevice;
use crate::nre_pipeline::NrePipeline;
use crate::nre_swap_chain::NreSwapChain;
use ash::vk;

pub struct NreRenderer {
    swap_chain: NreSwapChain,
    pipeline: NrePipeline,
    command_buffers: Vec<vk::CommandBuffer>,
    current_image_index: u32,
    current_frame_index: usize,
    is_frame_started: bool,
}

impl NreRenderer {
    pub fn new(
        device: &NreDevice,
        extent: vk::Extent2D,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Self {
        let swap_chain = NreSwapChain::new(device, extent);
        let pipeline = NrePipeline::new(device, swap_chain.render_pass(), descriptor_set_layout);
        let command_buffers = Self::create_command_buffers(device);
        Self {
            swap_chain,
            pipeline,
            command_buffers,
            current_image_index: 0,
            current_frame_index: 0,
            is_frame_started: false,
        }
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline.pipeline()
    }

    pub fn begin_frame(&mut self, device: &NreDevice) -> Option<vk::CommandBuffer> {
        let frame = self.current_frame_index;

        unsafe {
            device
                .device()
                .wait_for_fences(&[self.swap_chain.in_flight_fences[frame]], true, u64::MAX)
                .unwrap();
            device
                .device()
                .reset_fences(&[self.swap_chain.in_flight_fences[frame]])
                .unwrap();
        }

        let result = unsafe {
            self.swap_chain.swapchain_loader.acquire_next_image(
                self.swap_chain.swapchain,
                u64::MAX,
                self.swap_chain.image_available_semaphores[frame],
                vk::Fence::null(),
            )
        };

        let image_index = match result {
            Ok((index, _)) => index,
            Err(_) => return None,
        };

        self.current_image_index = image_index;
        self.is_frame_started = true;

        let cmd = self.get_current_command_buffer();

        unsafe {
            device
                .device()
                .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
                .unwrap();

            let begin_info = vk::CommandBufferBeginInfo::default();
            device
                .device()
                .begin_command_buffer(cmd, &begin_info)
                .unwrap();
        }

        Some(cmd)
    }

    pub fn end_frame(&mut self, device: &NreDevice) {
        let frame = self.current_frame_index;
        let cmd = self.get_current_command_buffer();

        unsafe {
            device.device().end_command_buffer(cmd).unwrap();
        }

        let wait_semaphores = [self.swap_chain.image_available_semaphores[frame]];
        let signal_semaphores =
            [self.swap_chain.render_finished_semaphores[self.current_image_index as usize]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let cmds = [cmd];

        let submit_info = vk::SubmitInfo {
            wait_semaphore_count: 1,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: cmds.as_ptr(),
            signal_semaphore_count: 1,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };

        unsafe {
            device
                .device()
                .queue_submit(
                    device.graphics_queue(),
                    &[submit_info],
                    self.swap_chain.in_flight_fences[frame],
                )
                .unwrap();
        }

        let swapchains = [self.swap_chain.swapchain];
        let indices = [self.current_image_index];
        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: indices.as_ptr(),
            ..Default::default()
        };

        unsafe {
            self.swap_chain
                .swapchain_loader
                .queue_present(device.graphics_queue(), &present_info)
                .unwrap();
        }

        self.current_frame_index = (self.current_frame_index + 1) % 2;
        self.is_frame_started = false;
    }

    pub fn begin_render_pass(&self, cmd: vk::CommandBuffer, device: &NreDevice) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.01, 0.01, 0.01, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_info = vk::RenderPassBeginInfo {
            render_pass: self.swap_chain.render_pass(),
            framebuffer: self.swap_chain.framebuffers[self.current_image_index as usize],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swap_chain.extent,
            },
            clear_value_count: 2,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.swap_chain.extent.width as f32,
            height: self.swap_chain.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swap_chain.extent,
        };

        unsafe {
            device.device().cmd_begin_render_pass(
                cmd,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            device.device().cmd_set_viewport(cmd, 0, &[viewport]);
            device.device().cmd_set_scissor(cmd, 0, &[scissor]);
        }
    }

    pub fn end_render_pass(&self, cmd: vk::CommandBuffer, device: &NreDevice) {
        unsafe {
            device.device().cmd_end_render_pass(cmd);
        }
    }

    pub fn get_current_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffers[self.current_frame_index]
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

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline.pipeline_layout()
    }

    pub fn render_pass(&self) -> vk::RenderPass {
        self.swap_chain.render_pass()
    }

    pub fn current_frame_index(&self) -> usize {
        self.current_frame_index
    }
}
