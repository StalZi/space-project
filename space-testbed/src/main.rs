#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

mod app;
use app::App;
use space_engine::logger::Logger;
use winit::event_loop::EventLoop;

const LOG: bool = true;
const LOG_VERBOSE: bool = true;
const LOG_KEYBOARD: bool = true;
const LOG_MOUSE: bool = true;
const LOG_PHYSICS: bool = false;

fn main() {
    Logger::create(LOG, LOG_VERBOSE, LOG_KEYBOARD, LOG_MOUSE, LOG_PHYSICS);

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
