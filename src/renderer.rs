use crate::diagnostic::FrameQueue;
use crate::egui_tools::EguiRenderer;
use crate::simulation::SimulationState;
use egui_wgpu::{
    wgpu::{self, util::RenderEncoder},
    ScreenDescriptor,
};
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;
use winit::window::Window;

pub enum RenderMode {
    Default,
    Trail,
}

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub scale_factor: f32,
    pub egui_renderer: EguiRenderer,
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub render_frames: Arc<Mutex<FrameQueue>>,
    pub render_texture_a: wgpu::TextureView,
    pub render_texture_b: wgpu::TextureView,
    pub use_target_a: bool,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub screen_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
    ) -> Self {
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
            present_mode: wgpu::PresentMode::AutoVsync,
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
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/particle.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(8),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::Zero,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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

        let render_texture_a = create_render_texture(&device, surface_config.format, width, height);
        let render_texture_b = create_render_texture(&device, surface_config.format, width, height);
        let use_target_a = true;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0, // Default; adjust based on mipmap usage
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let screen_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fullscreen Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/fullscreen_texture.wgsl").into()),
        });

        let screen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Screen Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let screen_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Screen Render Pipeline"),
            layout: Some(&screen_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &screen_shader_module,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &screen_shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let render_frames = Arc::new(Mutex::new(FrameQueue::new(300)));

        Self {
            device,
            queue,
            surface_config,
            surface,
            scale_factor,
            egui_renderer,
            pipeline,
            uniform_buffer,
            instance_buffer,
            vertex_buffer,
            render_frames,
            render_texture_a,
            render_texture_b,
            use_target_a,
            bind_group_layout,
            sampler,
            screen_pipeline,
        }
    }

    pub fn render(&mut self, window: &Window, simulation_state: &SimulationState) {
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: window.scale_factor() as f32 * self.scale_factor,
        };

        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // egui pass
        {
            self.egui_renderer.begin_frame(window);

            egui::Window::new("winit + egui + wgpu says hello!")
                .resizable(true)
                .vscroll(true)
                .default_open(false)
                .show(self.egui_renderer.context(), |ui| {
                    ui.label("Label!");

                    if ui.button("Button!").clicked() {
                        println!("boom!")
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Pixels per point: {}",
                            self.egui_renderer.context().pixels_per_point()
                        ));
                        if ui.button("-").clicked() {
                            self.scale_factor = (self.scale_factor - 0.1).max(0.3);
                        }
                        if ui.button("+").clicked() {
                            self.scale_factor = (self.scale_factor + 0.1).min(3.0);
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        /*
                        if let Ok(sim_frames) = simulation_frames.lock() {
                            ui.label(format!("Simulation FPS: {:.2}", sim_frames.calculate_fps()));
                        }
                        */
                        if let Ok(render_frames) = self.render_frames.lock() {
                            ui.label(format!("Render FPS: {:.2}", render_frames.calculate_fps()));
                        }
                    });
                });

            self.egui_renderer.end_frame_and_draw(
                &self.device,
                &self.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }

        // trail pass
        let target_texture = if self.use_target_a {
            &self.render_texture_a
        } else {
            &self.render_texture_b
        };

        let source_texture = if self.use_target_a {
            &self.render_texture_b
        } else {
            &self.render_texture_a
        };

        // nbody to texture pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Ping Pong Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_texture,
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

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Blend Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source_texture),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                ],
            });

            self.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&simulation_state.positions),
            );

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(0..6, 0..simulation_state.particle_count);
        }

        // texture to screen pass
        {
            let mut screen_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screen Render Pass"),
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

            let screen_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Screen Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(target_texture),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                ],
            });

            screen_render_pass.set_pipeline(&self.screen_pipeline);
            screen_render_pass.set_bind_group(0, &screen_bind_group, &[]);
            screen_render_pass.draw(0..6, 0..1);
        }

        self.use_target_a = !self.use_target_a;

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        {
            self.render_frames.lock().unwrap().record_frame();
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
        self.render_texture_a =
            create_render_texture(&self.device, self.surface_config.format, width, height);
        self.render_texture_b =
            create_render_texture(&self.device, self.surface_config.format, width, height);
        self.surface.configure(&self.device, &self.surface_config);
    }
}

fn create_render_texture(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> wgpu::TextureView {
    let texture_desc = wgpu::TextureDescriptor {
        label: Some("Render Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    device
        .create_texture(&texture_desc)
        .create_view(&Default::default())
}
