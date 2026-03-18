mod first_app;
mod nre_device;
mod nre_window;

use first_app::FirstApp;

fn main() {
    let app = FirstApp::new();
    app.run();
}
