use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::iter::once;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, color: [f32; 3]) -> Self {
        Self {
            position: [x, y, z],
            color,
        }
    }

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // 1.
            step_mode: wgpu::VertexStepMode::Vertex,                            // 2.
            attributes: &[
                // 3.
                wgpu::VertexAttribute {
                    offset: 0,                             // 4.
                    shader_location: 0,                    // 5.
                    format: wgpu::VertexFormat::Float32x3, // 6.
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

pub struct Entity {
    points: Vec<Vertex>,
    point_order: Vec<u32>,
}

impl Entity {
    pub fn from_points(points: Vec<Vertex>) -> Self {
        let mut order: Vec<u32> = vec![];
        for (i, _) in points.iter().enumerate() {
            order.push(i as u32);
        }
        Self {
            points,
            point_order: order,
        }
    }
}

struct WindowState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
}

fn get_device_limitations() -> wgpu::Limits {
    // if running on WebAssembly
    if cfg!(target_arch = "wasm32") {
        return wgpu::Limits::downlevel_webgl2_defaults();
    }
    wgpu::Limits::default()
}

fn get_wgpu_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::Backends::all())
}

fn create_gpu_context(window: &Window, gpu_handle: &wgpu::Instance) -> wgpu::Surface {
    unsafe { gpu_handle.create_surface(window) }
}

async fn generate_gpu_adapter(
    gpu_handle: &wgpu::Instance,
    surface: &wgpu::Surface,
) -> wgpu::Adapter {
    gpu_handle
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap()
}

async fn get_gpu_handle(gpu_adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    gpu_adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: get_device_limitations(),
                label: None,
            },
            None,
        )
        .await
        .unwrap()
}

fn get_context_configuration(
    window: &Window,
    surface: &wgpu::Surface,
    gpu_adapter: &wgpu::Adapter,
) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(gpu_adapter)[0],
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
    }
}

fn generate_shader_module(gpu: &wgpu::Device, file_as_str: &str) -> wgpu::ShaderModule {
    gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(file_as_str)),
    })
}

fn generate_render_pipeline_layout(gpu: &wgpu::Device) -> wgpu::PipelineLayout {
    gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    })
}

fn generate_render_pipeline(
    gpu: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    let layout = &generate_render_pipeline_layout(gpu);

    gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vert_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "frag_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

fn create_vertex_buffer(gpu: &wgpu::Device, buf_name: &str, vertices: &[Vertex]) -> wgpu::Buffer {
    gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(buf_name),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

fn create_index_buffer(gpu: &wgpu::Device, buf_name: &str, indices: &[u32]) -> wgpu::Buffer {
    gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(buf_name),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    })
}

impl WindowState {
    async fn new(window: Window) -> Self {
        let wgpu_handle = get_wgpu_instance();

        let surface = create_gpu_context(&window, &wgpu_handle);

        let gpu_adapter = generate_gpu_adapter(&wgpu_handle, &surface).await;

        let (gpu, gpu_work_queue) = get_gpu_handle(&gpu_adapter).await;

        let config = get_context_configuration(&window, &surface, &gpu_adapter);

        surface.configure(&gpu, &config);

        let shader = generate_shader_module(&gpu, include_str!("shader.wgsl"));

        let render_pipeline = generate_render_pipeline(&gpu, &config, &shader);

        Self {
            window,
            surface,
            device: gpu,
            queue: gpu_work_queue,
            config,
            render_pipeline,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&self) {}

    fn render(&self, entity: &Entity) -> Result<(), wgpu::SurfaceError> {
        let render_surface = self.surface.get_current_texture()?;
        let view = render_surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let vertex_buffer =
            create_vertex_buffer(&self.device, "Entity Vertex Buffer", &entity.points[..]);
        let index_buffer =
            create_index_buffer(&self.device, "Entity Index Buffer", &entity.point_order[..]);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline); // 2.
                                                             // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            //render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw(0..(entity.points.len() as u32), 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(once(encoder.finish()));
        render_surface.present();

        Ok(())
    }
}

pub fn to_srgb(rgb: f64) -> f64 {
    ((rgb / 255.0 + 0.055) / 1.055).powf(2.4)
}

pub struct Marlin {
    state: WindowState,
    event_loop: EventLoop<()>,
    entities: Vec<Entity>
}

impl Marlin {


    pub async fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        Self {
            state: WindowState::new(window).await,
            event_loop,
            entities: vec![]
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub async fn run(mut self) {

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::RedrawRequested(window_id) if window_id == self.state.window().id() => {
                    self.state.update();

                    for entity in &self.entities {
                        match self.state.render(entity) {
                            Ok(_) => {}
                            // Exit if the surface is lost
                            Err(wgpu::SurfaceError::Lost) => *control_flow = ControlFlow::Exit,
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    self.state.window().request_redraw();
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.state.window().id() => {
                    if !self.state.input(event) {
                        // UPDATED!
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(physical_size) => {
                                self.state.resize(*physical_size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                self.state.resize(**new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }

                _ => {}
            }
        });
    }




}
