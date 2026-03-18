use crate::nre_device::NreDevice;
use ash::vk;
use std::ffi::CString;

pub struct NrePipeline {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
}

impl NrePipeline {
    pub fn new(device: &NreDevice, render_pass: vk::RenderPass) -> Self {
        let pipeline_layout = Self::create_pipeline_layout(device.device());
        let pipeline = Self::create_pipeline(device.device(), render_pass, pipeline_layout);
        Self {
            pipeline,
            pipeline_layout,
        }
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    fn create_pipeline_layout(device: &ash::Device) -> vk::PipelineLayout {
        let layout_info = vk::PipelineLayoutCreateInfo {
            ..Default::default()
        };
        unsafe { device.create_pipeline_layout(&layout_info, None).unwrap() }
    }

    fn create_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        pipeline_layout: vk::PipelineLayout,
    ) -> vk::Pipeline {
        let vert_code = Self::read_shader("shaders/simple.vert.spv");
        let frag_code = Self::read_shader("shaders/simple.frag.spv");

        let vert_module = Self::create_shader_module(device, &vert_code);
        let frag_module = Self::create_shader_module(device, &frag_code);

        let entry_point = CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::VERTEX,
                module: vert_module,
                p_name: entry_point.as_ptr(),
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::FRAGMENT,
                module: frag_module,
                p_name: entry_point.as_ptr(),
                ..Default::default()
            },
        ];

        let vertex_input = vk::PipelineVertexInputStateCreateInfo {
            ..Default::default()
        };

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            scissor_count: 1,
            ..Default::default()
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            ..Default::default()
        };

        let multisampling = vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::RGBA,
            blend_enable: vk::FALSE,
            ..Default::default()
        };

        let color_blending = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: vk::FALSE,
            attachment_count: 1,
            p_attachments: &color_blend_attachment,
            ..Default::default()
        };

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo {
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input,
            p_input_assembly_state: &input_assembly,
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterizer,
            p_multisample_state: &multisampling,
            p_color_blend_state: &color_blending,
            p_dynamic_state: &dynamic_state,
            layout: pipeline_layout,
            render_pass,
            subpass: 0,
            ..Default::default()
        };

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .unwrap()[0]
        };

        unsafe {
            device.destroy_shader_module(vert_module, None);
            device.destroy_shader_module(frag_module, None);
        }

        pipeline
    }

    fn read_shader(path: &str) -> Vec<u32> {
        let bytes = std::fs::read(path).unwrap();
        ash::util::read_spv(&mut std::io::Cursor::new(bytes)).unwrap()
    }

    fn create_shader_module(device: &ash::Device, code: &[u32]) -> vk::ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo {
            code_size: code.len() * 4,
            p_code: code.as_ptr(),
            ..Default::default()
        };
        unsafe { device.create_shader_module(&create_info, None).unwrap() }
    }
}
