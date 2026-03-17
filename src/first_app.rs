use crate::nre_window::NreWindow;
use winit::event::{Event, WindowEvent};

pub struct FirstApp {
    nre_window: NreWindow,
}

impl FirstApp {
    pub fn new() -> Self {
        let nre_window = NreWindow::new(800, 600, "Nemesis Rendering Engine");
        Self { nre_window }
    }

    pub fn run(self) {
        let event_loop = self.nre_window.event_loop;
        event_loop
            .run(move |event, elwt| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    elwt.exit();
                }
                _ => {}
            })
            .unwrap();
    }
}
