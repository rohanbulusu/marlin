
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window}
};

use std::collections::HashMap;

use crate::entities::{Entity, Vertex, EntityBuilder, ShapeKind};
// use crate::colors::{RED, BLUE};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneName {
    Home,
    RootPicker,
    Grapher,
    Simulation
}

#[allow(clippy::from_over_into)]
impl Into<String> for SceneName {
    fn into(self) -> String {
        match self {
            Self::Home => "Home".to_string(),
            Self::RootPicker => "RootPicker".to_string(),
            Self::Grapher => "Grapher".to_string(),
            Self::Simulation => "Simulation".to_string()
        }
    }
}

pub struct ButtonBounds {
    horizontal: f32,
    vertical: f32
}

impl ButtonBounds {

    pub fn new(horizontal: f32, vertical: f32) -> ButtonBounds {
        Self { horizontal, vertical }
    }

}

pub struct VirtualButton {
    inhabiting_scene: SceneName,
    center: Vertex,
    scene_request: SceneName,
    bounds: ButtonBounds
}

impl VirtualButton {

    pub fn new(inhabiting_scene: SceneName, center: Vertex, scene_request: SceneName, width: f32, height: f32) -> VirtualButton {
        Self {
            inhabiting_scene,
            center,
            scene_request,
            bounds: ButtonBounds::new(width, height)
        }
    }

    pub fn left_bound(&self) -> f64 {
        (self.center.position[0] - self.bounds.horizontal / 3.5) as f64
    }

    pub fn right_bound(&self) -> f64 {
        (self.center.position[0] + self.bounds.horizontal / 3.5) as f64
    }

    pub fn top_bound(&self) -> f64 {
        (self.center.position[1] + self.bounds.vertical / 3.5) as f64
    }

    pub fn bottom_bound(&self) -> f64 {
        (self.center.position[1] - self.bounds.vertical / 3.5) as f64
    }

}

pub struct MousePosition {
    x: f64,
    y: f64,
    window_dimensions: (f64, f64)
}

impl MousePosition {

    pub fn new(x: f64, y: f64, window_width: f64, window_height: f64) -> MousePosition {
        Self {
            x, y,
            window_dimensions: (window_width, window_height)
        }
    }

    pub fn update_window_dimensions(&mut self, horizontal: f64, vertical: f64) {
        self.window_dimensions = (horizontal, vertical);
    }

    pub fn update_from_canvas_coords(&mut self, new_x: f64, new_y: f64) {
        let corrected_x = new_x + self.window_dimensions.0 / 2.0;
        let corrected_y = new_y + self.window_dimensions.1 / 2.0;
        self.x = corrected_x;
        self.y = corrected_y;
    }

    pub fn update_from_window_coords(&mut self, new_x: f64, new_y: f64) {
        self.x = new_x;
        self.y = new_y;
    }

    pub fn canvas_x(&self) -> f64 {
        self.x - self.window_dimensions.0 / 2.0
    }

    pub fn canvas_y(&self) -> f64 {
        -(self.y - self.window_dimensions.1 / 2.0)
    }

    fn within_horizontal_bounds(&self, x_left: f64, x_right: f64) -> bool {
        self.canvas_x() >= x_left && self.canvas_x() <= x_right
    }

    fn within_vertical_bounds(&self, y_bottom: f64, y_top: f64) -> bool {
        self.canvas_y() >= y_bottom && self.canvas_y() <= y_top
    }

    pub fn between(&self, x_left: f64, x_right: f64, y_bottom: f64, y_top: f64) -> bool {
        self.within_horizontal_bounds(x_left, x_right) && self.within_vertical_bounds(y_bottom, y_top)
    }

}


pub struct MasterWindowState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    cur_scene: SceneName,
    buttons: Vec<VirtualButton>,
    scenes: HashMap<SceneName, Vec<Entity>>,
    mouse_position: MousePosition
}

impl MasterWindowState {

    pub async fn new(window: Window) -> MasterWindowState {

        let size = window.inner_size();
        
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            // wgpu::Backends::all() => Vulkan + Metal + DX12 + WebGPU
            backends: wgpu::Backends::all(),
            // default shader compiler => naga
            dx12_shader_compiler: Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                // HighPerformance => more performant device, but more power-hungry
                power_preference: wgpu::PowerPreference::HighPerformance,
                // Some(&surface) => matches `surface` to the GPU
                compatible_surface: Some(&surface),
                // false => forces rendering system to use the GPU and not a 
                //          fallback system of any kind
                force_fallback_adapter: false
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                // specifies extra features potentially available on the GPU
                features: wgpu::Features::empty(),
                // the general limits on the types of resources able to be requested
                limits: wgpu::Limits::default(),
                label: Some("Local GPU Device")
            },
            None
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
                                                 .copied()
                                                 .find(|f| !f.describe().srgb)
                                                 .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![]
        };

        surface.configure(&device, &config);

        let mut scenes = HashMap::with_capacity(4);
        scenes.insert(SceneName::Home, vec![]);
        scenes.insert(SceneName::RootPicker, vec![]);
        scenes.insert(SceneName::Grapher, vec![]);
        scenes.insert(SceneName::Simulation, vec![]);

        let mouse_position = MousePosition::new(0.0, 0.0, size.width.into(), size.height.into());

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            cur_scene: SceneName::Home,
            buttons: vec![],
            scenes,
            mouse_position
        }

    }

    fn leftmost_value(vertices: &Vec<Vertex>) -> f32 {
        let mut leftmost = &vertices[0];
        for vertex in &vertices[1..vertices.len()] {
            if vertex.position[0] < leftmost.position[0] {
                leftmost = vertex;
            }
        }
        leftmost.position[0]
    }

    fn rightmost_value(vertices: &Vec<Vertex>) -> f32 {
        let mut rightmost = &vertices[0];
        for vertex in &vertices[1..vertices.len()] {
            if vertex.position[0] > rightmost.position[0] {
                rightmost = vertex;
            }
        }
        rightmost.position[0]
    }

    fn topmost_value(vertices: &Vec<Vertex>) -> f32 {
        let mut topmost = &vertices[0];
        for vertex in &vertices[1..vertices.len()] {
            if vertex.position[1] > topmost.position[1] {
                topmost = vertex;
            }
        }
        topmost.position[1]
    }

    fn bottommost_value(vertices: &Vec<Vertex>) -> f32 {
        let mut bottommost = &vertices[0];
        for vertex in &vertices[1..vertices.len()] {
            if vertex.position[1] < bottommost.position[1] {
                bottommost = vertex;
            }
        }
        bottommost.position[1]
    }

    pub fn add_button(&mut self, scene: &SceneName, shape: &ShapeKind, vertices: Vec<Vertex>, scene_request: SceneName) {
        
        let button_x = vertices.iter().map(|v| v.position[0]).sum::<f32>() / shape.requisite_points() as f32;
        let button_y = vertices.iter().map(|v| v.position[1]).sum::<f32>() / shape.requisite_points() as f32;

        let button = VirtualButton::new(
            *scene,
            Vertex::new(button_x, button_y, 0.0, vertices[0].color.into()),
            scene_request,
            (Self::leftmost_value(&vertices) - Self::rightmost_value(&vertices)).abs(),
            (Self::topmost_value(&vertices) - Self::bottommost_value(&vertices)).abs()
        );

        self.buttons.push(button);
        self.add_shape(scene, shape, vertices);
    }

    pub fn next_scene(&self) -> SceneName {
        match self.cur_scene {
            SceneName::Home => SceneName::RootPicker,
            SceneName::RootPicker => SceneName::Grapher,
            SceneName::Grapher => SceneName::Simulation,
            SceneName::Simulation => SceneName::Home
        }
    }

    pub fn previous_scene(&self) -> SceneName {
        match self.cur_scene {
            SceneName::Home => SceneName::Home,
            SceneName::RootPicker => SceneName::Home,
            SceneName::Grapher => SceneName::RootPicker,
            SceneName::Simulation => SceneName::Grapher
        }
    }

    pub fn add_entity(&mut self, scene: &SceneName, entity: Entity) {
        self.scenes.get_mut(scene).unwrap().push(entity);
    }

    pub fn add_shape(&mut self, scene: &SceneName, kind: &ShapeKind, vertices: Vec<Vertex>) {
        self.scenes.get_mut(scene).unwrap().push(
            EntityBuilder::from_shape(
                *kind,
                vertices
            ).unwrap().build(
                &self.device, 
                &self.config, 
                self.size.width,
                self.size.height
            )
        )
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn input(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position.update_from_window_coords(position.x, position.y);
            },
            WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                if *button != MouseButton::Left {
                    return;
                }
                let current_scene = self.cur_scene;
                for button in self.buttons.iter().filter(|b| b.inhabiting_scene == current_scene) {
                    if self.mouse_position.between(button.left_bound(), button.right_bound(), button.bottom_bound(), button.top_bound()) {
                        self.cur_scene = button.scene_request;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        for entity in self.scenes.get(&self.cur_scene).unwrap() {

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder")
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true
                        }
                    })],
                    depth_stencil_attachment: None
                });

                render_pass.set_pipeline(entity.pipeline());
                // println!("{}", entity.num_vertices());
                render_pass.set_vertex_buffer(0, entity.vertices().slice(..));
                render_pass.draw(0..entity.num_vertices(), 0..1);

            }

            self.queue.submit(std::iter::once(encoder.finish()));

        }

        output.present();

        Ok(())
    }

    pub async fn run(mut self, event_loop: EventLoop<()>) {
        env_logger::init();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => self.resize(physical_size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self.resize(*new_inner_size),
                _ => self.input(&event),
            },
            Event::RedrawRequested(_) => {
                self.update();
                match self.render() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => self.resize(self.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e)
                }
            },
            Event::MainEventsCleared => self.window().request_redraw(),
            _ => {}
        })
    }

}





