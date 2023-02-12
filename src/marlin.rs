
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use winit::window::Window;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // 1.
            step_mode: wgpu::VertexStepMode::Vertex, // 2.
            attributes: &[ // 3.
                wgpu::VertexAttribute {
                    offset: 0, // 4.
                    shader_location: 0, // 5.
                    format: wgpu::VertexFormat::Float32x3, // 6.
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }

    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

const INDICES: &[u32] = &[1, 2, 3]; 

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer
}

fn get_device_limitations() -> wgpu::Limits {
    // if running on WebAssembly
    if cfg!(target_arch = "wasm32") {
        return wgpu::Limits::downlevel_webgl2_defaults();
    }
    return wgpu::Limits::default();
}

fn get_wgpu_instance() -> wgpu::Instance {
        wgpu::Instance::new(wgpu::Backends::all())
}

fn create_gpu_context(window: &Window, gpu_handle: &wgpu::Instance) -> wgpu::Surface {
    unsafe { 
        gpu_handle.create_surface(window) 
    }
}

async fn generate_gpu_adapter(gpu_handle: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
    gpu_handle.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        },
    ).await.unwrap()
}

async fn get_gpu_handle(gpu_adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    gpu_adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: get_device_limitations(),
            label: None,
        },
        None
    ).await.unwrap()
}

fn get_context_configuration(window: &Window, surface: &wgpu::Surface, gpu_adapter: &wgpu::Adapter) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&gpu_adapter)[0],
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

fn generate_render_pipeline(gpu: &wgpu::Device, config: &wgpu::SurfaceConfiguration, shader: &wgpu::ShaderModule) -> wgpu::RenderPipeline {
    
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
            module: &shader,
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
        multiview: None, // 5.
    })
}

fn create_vertex_buffer(gpu: &wgpu::Device, buf_name: &str, vertices: &[Vertex]) -> wgpu::Buffer {
    gpu.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some(buf_name),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }
    )
}

fn create_index_buffer(gpu: &wgpu::Device, buf_name: &str, indices: &[u32]) -> wgpu::Buffer {
    gpu.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some(buf_name),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        }
    )
}

impl State {

    async fn new(window: Window) -> Self {

        let wgpu_handle = get_wgpu_instance();
        
        let surface = create_gpu_context(&window, &wgpu_handle);

        let gpu_adapter = generate_gpu_adapter(&wgpu_handle, &surface).await;

        let (gpu, gpu_work_queue) = get_gpu_handle(&gpu_adapter).await;

        let config = get_context_configuration(&window, &surface, &gpu_adapter);

        surface.configure(&gpu, &config);

        let shader = generate_shader_module(&gpu, include_str!("shader.wgsl"));

        let render_pipeline = generate_render_pipeline(&gpu, &config, &shader);

        let vertex_buffer = create_vertex_buffer(&gpu, "Vertex Buffer", VERTICES);

        let index_buffer = create_index_buffer(&gpu, "Index Buffer", INDICES);

        Self {
            window,
            surface,
            device: gpu,
            queue: gpu_work_queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer
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

    fn update(&mut self) {
        
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
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
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw(0..3, 0..1);
            render_pass.draw(0..(VERTICES.len() as u32), 0..1); // 3.
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub fn to_srgb(rgb: f64) -> f64 {
    ((rgb / 255.0 + 0.055) / 1.055).powf(2.4)
}


pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.window.inner_size()),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) { // UPDATED!
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
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    });
}