use crate::nre_descriptor::{NreDescriptorPool, NreDescriptorSetLayout, NreUniformBuffer};
use crate::nre_device::NreDevice;
use crate::nre_model::NreModel;
use crate::nre_renderer::NreRenderer;
use crate::nre_window::NreWindow;
use ash::vk;
use glam;
use winit::event::{Event, WindowEvent};

pub struct PushConstantData {
    offset: [f32; 2],
    scale: [f32; 2],
}

pub struct FirstApp {
    nre_window: NreWindow,
    nre_device: NreDevice,
    nre_renderer: NreRenderer,
    nre_model: NreModel,
    start_time: std::time::Instant,
    descriptor_set_layout: NreDescriptorSetLayout,
    descriptor_pool: NreDescriptorPool,
    uniform_buffers: NreUniformBuffer,
    descriptor_sets: Vec<vk::DescriptorSet>,
}

impl FirstApp {
    pub fn new() -> Self {
        //
        let nre_window = NreWindow::new(800, 600, "Nemesis Rendering Engine");

        let nre_device = NreDevice::new(&nre_window.window);

        let extent = vk::Extent2D {
            width: 800,
            height: 600,
        };

        let descriptor_set_layout = NreDescriptorSetLayout::new(&nre_device);
        let descriptor_pool = NreDescriptorPool::new(&nre_device);
        let uniform_buffers = NreUniformBuffer::new(&nre_device);
        let descriptor_sets =
            descriptor_pool.allocate_descriptor_sets(&nre_device, descriptor_set_layout.layout());

        let nre_renderer = NreRenderer::new(&nre_device, extent, descriptor_set_layout.layout());

        let vertices = vec![
            crate::nre_model::Vertex {
                position: [0.0, -0.5],
            },
            crate::nre_model::Vertex {
                position: [0.5, 0.5],
            },
            crate::nre_model::Vertex {
                position: [-0.5, 0.5],
            },
        ];

        let nre_model = NreModel::new(&nre_device, &vertices);

        let start_time = std::time::Instant::now();

        for i in 0..2 {
            let buffer_info = vk::DescriptorBufferInfo {
                buffer: uniform_buffers.buffer(i),
                offset: 0,
                range: std::mem::size_of::<glam::Mat4>() as u64,
            };
            let write = vk::WriteDescriptorSet {
                dst_set: descriptor_sets[i],
                dst_binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_buffer_info: &buffer_info,
                ..Default::default()
            };
            unsafe { nre_device.device().update_descriptor_sets(&[write], &[]) };
        }

        Self {
            nre_window,
            nre_device,
            nre_renderer,
            nre_model,
            start_time,
            descriptor_set_layout,
            descriptor_pool,
            uniform_buffers,
            descriptor_sets,
        }
    }

    pub fn run(mut self) {
        let event_loop = self.nre_window.event_loop;
        event_loop
            .run(move |event, elwt| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    elwt.exit();
                }
                Event::AboutToWait => {
                    if let Some(cmd) = self.nre_renderer.begin_frame(&self.nre_device) {
                        self.nre_renderer.begin_render_pass(cmd, &self.nre_device);
                        let time = self.start_time.elapsed().as_secs_f32();
                        let matrix = glam::Mat4::from_rotation_z(time);
                        let frame = self.nre_renderer.current_frame_index();
                        unsafe {
                            let ptr = self.uniform_buffers.mapped_ptr(frame) as *mut glam::Mat4;
                            ptr.write(matrix);

                            self.nre_device.device().cmd_bind_pipeline(
                                cmd,
                                ash::vk::PipelineBindPoint::GRAPHICS,
                                self.nre_renderer.pipeline(),
                            );

                            self.nre_device.device().cmd_bind_descriptor_sets(
                                cmd,
                                vk::PipelineBindPoint::GRAPHICS,
                                self.nre_renderer.pipeline_layout(),
                                0,
                                &[self.descriptor_sets[self.nre_renderer.current_frame_index()]],
                                &[],
                            );

                            let push_data = PushConstantData {
                                offset: [0.0, 0.0],
                                scale: [1.0, 1.0],
                            };
                            let push_bytes = std::slice::from_raw_parts(
                                &push_data as *const PushConstantData as *const u8,
                                std::mem::size_of::<PushConstantData>(),
                            );
                            self.nre_device.device().cmd_push_constants(
                                cmd,
                                self.nre_renderer.pipeline_layout(),
                                vk::ShaderStageFlags::VERTEX,
                                0,
                                push_bytes,
                            );

                            self.nre_device.device().cmd_bind_vertex_buffers(
                                cmd,
                                0,
                                &[self.nre_model.vertex_buffer()],
                                &[0],
                            );
                            self.nre_device.device().cmd_draw(
                                cmd,
                                self.nre_model.vertex_count(),
                                1,
                                0,
                                0,
                            );
                        }
                        self.nre_renderer.end_render_pass(cmd, &self.nre_device);
                        self.nre_renderer.end_frame(&self.nre_device);
                    }
                }
                _ => {}
            })
            .unwrap();
    }
}
