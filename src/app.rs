use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};
use egui_wgpu::wgpu;
use winit::dpi::PhysicalSize;
use crate::renderer::Renderer;
use crate::simulation::SimulationState;
use crate::diagnostic::FrameQueue;

pub struct App {
    instance: wgpu::Instance,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    simulation: Arc<Mutex<SimulationState>>,
    frame_queue: FrameQueue,
}

impl App {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let simulation = Arc::new(Mutex::new(SimulationState::new(100)));
        Self {
            instance,
            window: None,
            renderer: None,
            simulation,
            frame_queue: FrameQueue::new(60),
        }
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        let initial_width = 1360;
        let initial_height = 768;

        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let renderer = Renderer::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_width,
        )
        .await;

        self.window.get_or_insert(window);
        self.renderer.get_or_insert(renderer);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        self.renderer.as_mut().unwrap().resize_surface(width, height);
    }

    fn handle_redraw(&mut self) {
        let now = Instant::now();
        let dt = (now - self.frame_queue.get_last_frame().unwrap_or(now)).as_secs_f32();
        self.frame_queue.record_frame();

        let time_step = 0.1;
        self.simulation.lock().unwrap().update(time_step);
        
        self.renderer.as_mut().unwrap().render(
            self.window.as_ref().unwrap(), 
            &self.simulation.lock().unwrap()
        );
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        pollster::block_on(self.set_window(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        self.renderer
            .as_mut()
            .unwrap()
            .egui_renderer
            .handle_input(self.window.as_ref().unwrap(), &event);

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw();
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            _ => (),
        }
    }
}
