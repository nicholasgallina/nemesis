use crate::first_app::PushConstantData;
use crate::nre_device::NreDevice;
use crate::nre_model::Vertex;
use ash::vk;
use std::ffi::CString;

pub struct NrePipeline {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    device: ash::Device,
}

impl NrePipeline {
    pub fn new(
        device: &NreDevice,
        render_pass: vk::RenderPass,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Self {
        let pipeline_layout = Self::create_pipeline_layout(device.device(), descriptor_set_layout);
        let pipeline = Self::create_pipeline(device.device(), render_pass, pipeline_layout);
        Self {
            pipeline,
            pipeline_layout,
            device: device.device().clone(),
        }
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    fn create_pipeline_layout(
        device: &ash::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> vk::PipelineLayout {
        let push_constant_range = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: std::mem::size_of::<PushConstantData>() as u32,
        };
        let layout_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: &descriptor_set_layout,
            push_constant_range_count: 1,
            p_push_constant_ranges: &push_constant_range,
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

        let binding_descriptions = Vertex::get_binding_descriptions();
        let attribute_descriptions = Vertex::get_attribute_descriptions();
        let vertex_input = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: binding_descriptions.len() as u32,
            p_vertex_binding_descriptions: binding_descriptions.as_ptr(),
            vertex_attribute_description_count: attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: attribute_descriptions.as_ptr(),
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

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: vk::FALSE,
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
            p_depth_stencil_state: &depth_stencil,
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

// overide! DROP
impl Drop for NrePipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
