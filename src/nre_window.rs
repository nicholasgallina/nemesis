use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct NreWindow {
    pub event_loop: EventLoop<()>,
    pub window: Window,
    width: u32,
    height: u32,
    name: String,
}

impl NreWindow {
    pub fn new(width: u32, height: u32, name: &str) -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new()
            .with_title("Nemesis")
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        Self {
            event_loop,
            window,
            width,
            height,
            name: name.to_string(),
        }
    }
}
