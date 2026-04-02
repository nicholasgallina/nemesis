use crate::nre_device::NreDevice;
use crate::nre_model::NreModel;
use crate::nre_renderer::NreRenderer;
use crate::nre_window::NreWindow;
use ash::vk;
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

        let nre_renderer = NreRenderer::new(&nre_device, extent);

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

        Self {
            nre_window,
            nre_device,
            nre_renderer,
            nre_model,
            start_time,
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
                        unsafe {
                            self.nre_device.device().cmd_bind_pipeline(
                                cmd,
                                ash::vk::PipelineBindPoint::GRAPHICS,
                                self.nre_renderer.pipeline(),
                            );

                            let push_data = PushConstantData {
                                offset: [time.sin() * 0.5, time.cos() * 0.5],
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
