mod first_app;
mod nre_descriptor;
mod nre_device;
mod nre_model;
mod nre_pipeline;
mod nre_renderer;
mod nre_swap_chain;
mod nre_window;

use first_app::FirstApp;

fn main() {
    let app = FirstApp::new();
    app.run();
}
