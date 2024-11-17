use crate::buffer_tools::TripleBuffer;
use crate::egui_tools::EguiRenderer;
use egui_wgpu::wgpu::util::DeviceExt;
use egui_wgpu::{wgpu, ScreenDescriptor};
use rand::Rng;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub fn start_simulation_thread(
    state_buffer: Arc<TripleBuffer<SimulationState>>,
    simulation_frames: Arc<Mutex<FrameQueue>>,
) {
    std::thread::spawn(move || {
        let mut last_time = std::time::Instant::now();

        loop {
            let now = std::time::Instant::now();
            let dt = now.duration_since(last_time).as_secs_f32();
            last_time = now;

            log::info!("Simulation update dt: {:.6} seconds", dt);

            {
                let mut simulation_state = state_buffer.get_write_buffer();
                simulation_state.update(0.1);
            }

            // commit after we drop the lock
            state_buffer.commit();

            {
                simulation_frames.lock().unwrap().record_frame();
            }
        }
    });
}

pub struct FrameQueue {
    frames: VecDeque<Instant>,
    max_frames: usize,
}

impl FrameQueue {
    pub fn new(max_frames: usize) -> Self {
        Self {
            frames: VecDeque::new(),
            max_frames,
        }
    }

    pub fn record_frame(&mut self) {
        let timestamp = Instant::now();

        self.frames.push_back(timestamp);

        if self.frames.len() > self.max_frames {
            self.frames.pop_front();
        }
    }

    pub fn calculate_fps(&self) -> f32 {
        if self.frames.len() < 2 {
            return 0.0;
        }

        let first = *self.frames.front().unwrap();
        let last = *self.frames.back().unwrap();
        let duration = (last - first).as_secs_f32();

        self.frames.len() as f32 / duration
    }
}

#[derive(Clone)]
pub struct SimulationState {
    pub particle_count: u32,
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
    pub accelerations: Vec<[f32; 3]>,
    pub masses: Vec<f32>,
    pub g: f32,
}
impl SimulationState {
    pub fn new(particle_count: u32) -> Self {
        let mut rng = rand::thread_rng();
        let positions: Vec<[f32; 3]> = (0..particle_count)
            .map(|_| {
                let r = f32::powf(rng.gen_range(0.0..0.9), 0.1);
                let theta = rng.gen_range(0.0..std::f32::consts::TAU);
                // phi stuff is for 3d
                // let phi = rng.gen_range(0.0..std::f32::consts::PI);

                // Convert spherical coordinates to cartesian
                let x = r * theta.cos(); // * phi.sin();
                let y = r * theta.sin(); // * phi.sin();
                let z = 0.0;
                // let z = r * phi.cos();

                [x, y, z]
            })
            .collect();
        let velocities: Vec<[f32; 3]> = positions
            .iter()
            .map(|[x, y, _]| {
                let speed = rng.gen_range(0.004..0.008);
                let magnitude = f32::sqrt(x * x + y * y);
                let radial_unit = [x / magnitude, y / magnitude, 0.0].map(|x| x * speed);

                [-radial_unit[1], radial_unit[0], 0.0]
            })
            .collect();

        let accelerations = vec![[0.0, 0.0, 0.0]; particle_count as usize];
        let masses = vec![1000.0; particle_count as usize];
        let g = 6.67430e-11;

        Self {
            particle_count,
            positions,
            velocities,
            accelerations,
            masses,
            g,
        }
    }

    fn update_acceleration(&mut self) {
        // F = (G*m1m2/(r*r)) * (unit vector)
        // F = ma
        // a = G*m2/(r*r) * unit vector
        let softening = 1e-4;

        for p1 in 0..self.particle_count as usize {
            let mut acceleration = [0.0, 0.0, 0.0];
            let p1_pos = self.positions[p1];

            for p2 in 0..self.particle_count as usize {
                if p1 == p2 {
                    continue;
                }

                let p2_pos = self.positions[p2];
                let p2_mass = self.masses[p2];

                let dx = p2_pos[0] - p1_pos[0];
                let dy = p2_pos[1] - p1_pos[1];
                let dz = p2_pos[2] - p1_pos[2];
                let dist_sqr = dx * dx + dy * dy + dz * dz + softening;

                let inv_dist = 1.0 / f32::sqrt(dist_sqr);
                let inv_dist3 = inv_dist * inv_dist * inv_dist;

                let force = self.g * p2_mass * inv_dist3;

                acceleration[0] += force * dx;
                acceleration[1] += force * dy;
                acceleration[2] += force * dz;
            }

            self.accelerations[p1][0] = acceleration[0];
            self.accelerations[p1][1] = acceleration[1];
            self.accelerations[p1][2] = acceleration[2];
        }
    }

    pub fn update(&mut self, dt: f32) {
        //self.update_euler(dt);
        self.update_leapfrog(dt);
    }

    #[allow(dead_code)]
    fn update_euler(&mut self, dt: f32) {
        self.update_acceleration();

        for p1 in 0..self.particle_count as usize {
            for i in 0..3 {
                self.velocities[p1][i] += self.accelerations[p1][i] * dt;
                self.positions[p1][i] += self.velocities[p1][i] * dt;
            }
        }
    }

    #[allow(dead_code)]
    fn update_leapfrog(&mut self, dt: f32) {
        for p1 in 0..self.particle_count as usize {
            for i in 0..3 {
                self.velocities[p1][i] += self.accelerations[p1][i] * dt * 0.5;
                self.positions[p1][i] += self.velocities[p1][i] * dt;
            }
        }

        self.update_acceleration();

        for p1 in 0..self.particle_count as usize {
            for i in 0..3 {
                self.velocities[p1][i] += self.accelerations[p1][i] * dt * 0.5;
            }
        }
    }
}

pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,
    pub nbody_pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub instance_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub state_buffer_render: Arc<TripleBuffer<SimulationState>>,
    pub render_frames: Arc<Mutex<FrameQueue>>,
    pub simulation_frames: Arc<Mutex<FrameQueue>>,
}

impl AppState {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
    ) -> Self {
        let particle_count = 1000;
        let simulation_state = Arc::new(TripleBuffer::new(SimulationState::new(particle_count)));
        let state_buffer_sim = Arc::clone(&simulation_state);
        let state_buffer_render = Arc::clone(&simulation_state);
        let simulation_frames = Arc::new(Mutex::new(FrameQueue::new(300)));

        start_simulation_thread(state_buffer_sim, Arc::clone(&simulation_frames));

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a suitable adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let surface_format = swapchain_capabilities
            .formats
            .iter()
            .find(|&&f| f == wgpu::TextureFormat::Bgra8UnormSrgb)
            .expect("Surface format not supported");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        // egui
        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, window);
        let scale_factor = 1.0;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("N-Body Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("particle.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(8),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let nbody_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("NBody Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: (std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress)
                            as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: (std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress)
                            as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Float32x3],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(surface_config.format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            // todo: update on resize?
            contents: bytemuck::cast_slice(&[
                window.inner_size().width,
                window.inner_size().height,
            ]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: None,
        });

        // Initialize particle buffer with dummy data
        let initial_particles = vec![[0.0f32, 0.0f32, 0.0f32]; 10000];
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Instance Buffer"),
            contents: bytemuck::cast_slice(&initial_particles),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let quad_vertices = [
            [-1.0, -1.0], // Bottom-left
            [1.0, -1.0],  // Bottom-right
            [1.0, 1.0],   // Top-right
            [-1.0, -1.0], // Bottom-left (reused)
            [1.0, 1.0],   // Top-right (reused)
            [-1.0, 1.0],  // Top-left
        ]
        .iter()
        .map(|x| {
            x.iter()
                .map(|x| x * 0.01)
                .collect::<Vec<f32>>()
                .try_into()
                .unwrap()
        })
        .collect::<Vec<[f32; 2]>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let render_frames = Arc::new(Mutex::new(FrameQueue::new(300)));

        Self {
            device,
            queue,
            surface_config,
            surface,
            scale_factor,
            egui_renderer,
            nbody_pipeline,
            uniform_buffer,
            bind_group,
            instance_buffer,
            vertex_buffer,
            state_buffer_render,
            render_frames,
            simulation_frames,
        }
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32]),
        );
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}

pub struct App {
    instance: wgpu::Instance,
    state: Option<AppState>,
    window: Option<Arc<Window>>,
}

impl App {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        Self {
            instance,
            state: None,
            window: None,
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

        let state = AppState::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_width,
        )
        .await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        self.state.as_mut().unwrap().resize_surface(width, height);
    }

    fn handle_redraw(&mut self) {
        let state = self.state.as_mut().unwrap();

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.surface_config.width, state.surface_config.height],
            pixels_per_point: self.window.as_ref().unwrap().scale_factor() as f32
                * state.scale_factor,
        };

        let surface_texture = state
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let window = self.window.as_ref().unwrap();

        // egui pass
        {
            state.egui_renderer.begin_frame(window);

            egui::Window::new("winit + egui + wgpu says hello!")
                .resizable(true)
                .vscroll(true)
                .default_open(false)
                .show(state.egui_renderer.context(), |ui| {
                    ui.label("Label!");

                    if ui.button("Button!").clicked() {
                        println!("boom!")
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Pixels per point: {}",
                            state.egui_renderer.context().pixels_per_point()
                        ));
                        if ui.button("-").clicked() {
                            state.scale_factor = (state.scale_factor - 0.1).max(0.3);
                        }
                        if ui.button("+").clicked() {
                            state.scale_factor = (state.scale_factor + 0.1).min(3.0);
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if let Ok(sim_frames) = state.simulation_frames.lock() {
                            ui.label(format!("Simulation FPS: {:.2}", sim_frames.calculate_fps()));
                        }
                        if let Ok(render_frames) = state.render_frames.lock() {
                            ui.label(format!("Render FPS: {:.2}", render_frames.calculate_fps()));
                        }
                    });
                });

            state.egui_renderer.end_frame_and_draw(
                &state.device,
                &state.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }

        // nbody pass
        {
            let simulation_state = state.state_buffer_render.get_read_buffer();

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("NBody Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            state.queue.write_buffer(
                &state.instance_buffer,
                0,
                bytemuck::cast_slice(&simulation_state.positions),
            );

            render_pass.set_pipeline(&state.nbody_pipeline);
            render_pass.set_bind_group(0, &state.bind_group, &[]);
            render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, state.instance_buffer.slice(..));
            render_pass.draw(0..6, 0..simulation_state.particle_count);
        }

        // swap read once we drop the buffer
        state.state_buffer_render.swap_read();

        state.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        {
            state.render_frames.lock().unwrap().record_frame();
        }
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
        self.state
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
