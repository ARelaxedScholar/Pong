#![warn(clippy::all, clippy::pedantic)]
use std::sync::{Arc, Mutex};

use glfw::{fail_on_errors, Action, Context, Key, Window};
use wgpu::{
    self,
    util::{DeviceExt, RenderEncoder},
    BackendOptions, Backends, BufferDescriptor, BufferUsages, Color, InstanceDescriptor,
    InstanceFlags, PipelineCompilationOptions, RenderPipelineDescriptor, RequestAdapterOptionsBase,
    VertexState, QUERY_RESOLVE_BUFFER_ALIGNMENT,
};

struct Player {
    vertices: Vec<Vertex>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    window: &'a mut glfw::Window,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (i32, i32),
    render_pipeline: wgpu::RenderPipeline,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a mut Window) -> Self {
        let size = window.get_size();
        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: Backends::VULKAN,
            flags: InstanceFlags::default(),
            backend_options: BackendOptions::default(),
        });

        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&window) }
            .expect("Failed to get target");
        let surface =
            unsafe { instance.create_surface_unsafe(target) }.expect("Failed to get surface");

        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase::default())
            .await
            .expect("Failed to get adapter");

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
            .expect("Failed to get device and queue");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let (width, height) = (size.0.max(1), size.1.max(1));
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: width as u32,
            height: height as u32,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Default Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        State {
            surface,
            device,
            window,
            queue,
            config,
            size: (width, height),
            render_pipeline,
        }
    }

    pub fn on_key_press(
        window: &mut glfw::Window,
        key: glfw::Key,
        _: i32,
        action: glfw::Action,
        _: glfw::Modifiers,
    ) {
    }
}

async fn run() {
    let mut glfw = glfw::init(fail_on_errors!()).expect("Failed to get glfw instance");

    let (mut window, events) = glfw
        .create_window(1_000, 600, "Pong", glfw::WindowMode::Windowed)
        .expect("Failed to get window and events handlers.");

    window.set_key_polling(true);
    window.make_current();

    let mut vertices_1 = [
        Vertex {
            position: [-0.8, 0.2, 0.0],
            color: [1., 1., 1.],
        }, // A
        Vertex {
            position: [-0.8, -0.2, 0.0],
            color: [1., 1., 1.],
        }, // B
        Vertex {
            position: [-0.77, 0.2, 0.0],
            color: [1., 1., 1.],
        }, // C
        Vertex {
            position: [-0.77, -0.2, 0.0],
            color: [1., 1., 1.],
        }, // D
    ];
    let indices_1: &[u16] = &[0, 1, 2, 2, 1, 3];

    let player_1 = Arc::new(Mutex::new(Player {
        vertices: Vec::from(vertices_1),
    }));

    let mut vertices_2 = [
        Vertex {
            position: [0.8, 0.2, 0.0],
            color: [1., 1., 1.],
        }, // A
        Vertex {
            position: [0.8, -0.2, 0.0],
            color: [1., 1., 1.],
        }, // B
        Vertex {
            position: [0.77, 0.2, 0.0],
            color: [1., 1., 1.],
        }, // C
        Vertex {
            position: [0.77, -0.2, 0.0],
            color: [1., 1., 1.],
        }, // D
    ];
    let indices_2: &[u16] = &[4, 6, 5, 6, 7, 5];
    let player_2 = Arc::new(Mutex::new(Player {
        vertices: Vec::from(vertices_2),
    }));

    let ball = &[
        Vertex {
            position: [0.02, 0.02, 0.],
            color: [1., 1., 1.],
        },
        Vertex {
            position: [-0.02, 0.02, 0.],
            color: [1., 1., 1.],
        },
        Vertex {
            position: [-0.02, -0.02, 0.],
            color: [1., 1., 1.],
        },
        Vertex {
            position: [0.02, -0.02, 0.],
            color: [1., 1., 1.],
        },
    ];

    let ball_indices: &[u16] = &[8, 9, 10, 8, 10, 11];

    let mut combined_vertices = vec![];
    combined_vertices.extend_from_slice(&player_1.lock().unwrap().vertices);
    combined_vertices.extend_from_slice(&player_2.lock().unwrap().vertices);
    combined_vertices.extend_from_slice(ball);

    let mut combined_indices = Vec::from(indices_1);
    combined_indices.extend_from_slice(indices_2);
    combined_indices.extend_from_slice(ball_indices);
    {
        let p1 = Arc::clone(&player_1);
        let p2 = Arc::clone(&player_2);
        window.set_key_callback(Box::new(
            move |window: &mut glfw::Window,
                  key: glfw::Key,
                  _: i32,
                  action: glfw::Action,
                  _: glfw::Modifiers| {
                if action == glfw::Action::Press || action == glfw::Action::Repeat {
                    match key {
                        glfw::Key::W => {
                            p1.lock()
                                .unwrap()
                                .vertices
                                .iter_mut()
                                .for_each(|vertex| vertex.position[1] += 0.05);
                        }
                        glfw::Key::S => {
                            p1.lock()
                                .unwrap()
                                .vertices
                                .iter_mut()
                                .for_each(|vertex| vertex.position[1] -= 0.05);
                        }
                        glfw::Key::Up => {
                            p2.lock()
                                .unwrap()
                                .vertices
                                .iter_mut()
                                .for_each(|vertex| vertex.position[1] += 0.05);
                        }
                        glfw::Key::Down => {
                            p2.lock()
                                .unwrap()
                                .vertices
                                .iter_mut()
                                .for_each(|vertex| vertex.position[1] -= 0.05);
                        }
                        _ => {}
                    }
                }
            },
        ));

        // Defining the vertex buffers
        //
    }

    let state = State::new(&mut window).await;

    let index_buffer = state
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(combined_indices.as_slice()),
            usage: BufferUsages::INDEX,
        });

    let vertex_buffer = state
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(combined_vertices.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
    while !state.window.should_close() {
        glfw.poll_events();

        // Update Buffer
        let mut new_vertices = vec![];
        new_vertices.extend_from_slice(&player_1.lock().unwrap().vertices);
        new_vertices.extend_from_slice(&player_2.lock().unwrap().vertices);
        new_vertices.extend_from_slice(ball);

        state.queue.write_buffer(
            &vertex_buffer,
            0,
            bytemuck::cast_slice(new_vertices.as_slice()),
        );

        // Rendering
        let output = state
            .surface
            .get_current_texture()
            .expect("Failed to get texture");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&state.render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..combined_indices.len() as u32, 0, 0..1);
        drop(render_pass);
        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

fn main() {
    pollster::block_on(run());
}
