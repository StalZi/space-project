#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

mod app;
use app::App;

use winit::event_loop::EventLoop;

use space_engine::logger::{Logger};

fn main() {
    if cfg!(debug_assertions) {
        Logger::create(true, true);
    } else {
        Logger::create(false, false);
    }

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
