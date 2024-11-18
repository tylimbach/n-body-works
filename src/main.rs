use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod buffer_tools;
mod diagnostic;
mod egui_tools;
mod renderer;
mod simulation;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}

async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = app::App::new();
    event_loop.run_app(&mut app).expect("Failed to run app");
}
