use crate::nre_device::NreDevice;
use crate::nre_renderer::NreRenderer;
use crate::nre_window::NreWindow;
use ash::vk;
use winit::event::{Event, WindowEvent};

pub struct FirstApp {
    nre_window: NreWindow,
    nre_device: NreDevice,
    nre_renderer: NreRenderer,
}

impl FirstApp {
    pub fn new() -> Self {
        let nre_window = NreWindow::new(800, 600, "Nemesis Rendering Engine");
        let nre_device = NreDevice::new(&nre_window.window);
        let extent = vk::Extent2D {
            width: 800,
            height: 600,
        };
        let nre_renderer = NreRenderer::new(&nre_device, extent);
        Self {
            nre_window,
            nre_device,
            nre_renderer,
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
                        println!("frame started");
                        self.nre_renderer.begin_render_pass(cmd, &self.nre_device);
                        unsafe {
                            self.nre_device.device().cmd_bind_pipeline(
                                cmd,
                                ash::vk::PipelineBindPoint::GRAPHICS,
                                self.nre_renderer.pipeline(),
                            );
                            self.nre_device.device().cmd_draw(cmd, 3, 1, 0, 0);
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
