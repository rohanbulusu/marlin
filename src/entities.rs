
use wgpu::util::DeviceExt;
use hebrides::linal::Vector;

use crate::colors::Color;


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub position: [f32; 3],
	pub color: [f32; 3]
}

impl Vertex {

    pub fn new(x: f32, y: f32, z: f32, color: Color) -> Vertex {
        Self {
            position: [x, y, z],
            color: color.in_percentages()
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3
                }
            ]
        }
    }

    pub fn as_vector(&self) -> Vector<f32> {
        Vector::new(self.position.to_vec())
    }

}

impl std::fmt::Display for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.position[0], self.position[1], self.position[2])
    }
}

impl From<Vertex> for Vector<f32> {
    fn from(value: Vertex) -> Self {
        Vector::new(value.position.to_vec())
    }
}

pub struct Entity {
    vertices: Vec<Vertex>,
    vertex_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline
}

impl Entity {

    pub fn new(gpu: &wgpu::Device, surface_configuration: &wgpu::SurfaceConfiguration, width: f32, height: f32, vertices: Vec<Vertex>) -> Entity {
        
        let points = Self::normalize_coordinates(&vertices, width, height);

        let vertex_buffer = gpu.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(points.as_slice()),
                usage: wgpu::BufferUsages::VERTEX
            }
        );

        let shader = gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
        });

        let render_pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[]
        });

        let render_pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex_shader_main",
                buffers: &[Vertex::desc()]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment_shader_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_configuration.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                })]
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
                alpha_to_coverage_enabled: false
            },
            multiview: None
        });

        Self { 
            vertices: points, 
            vertex_buffer, 
            render_pipeline 
        }
    }

    fn normalize_coordinates(vertices: &[Vertex], width: f32, height: f32) -> Vec<Vertex> {
        let mut normalized = Vec::with_capacity(vertices.len());
        for vertex in vertices {
            normalized.push(Vertex::new(
                vertex.position[0] / width,
                vertex.position[1] / height,
                vertex.position[2],
                vertex.color.into()
            ));
        }
        normalized
    }

    pub fn vertices(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    pub fn num_vertices(&self) -> u32 {
        self.vertices.len() as u32
    }

}

pub struct EntityBuilder {
    vertices: Vec<Vertex>
}

impl EntityBuilder {

    fn new(vertices: Vec<Vertex>) -> EntityBuilder {
        Self { vertices }
    }

    fn valid_vertex_number(kind: &ShapeKind, num_vertices: usize) -> Option<ShapeError> {
        match num_vertices.cmp(&kind.requisite_points()) {
            std::cmp::Ordering::Less => Some(ShapeError::VertexUnderspecification(*kind)),
            std::cmp::Ordering::Greater => Some(ShapeError::VertexOverspecification(*kind)),
            std::cmp::Ordering::Equal => None
        }
    }

    pub fn from_shape(kind: ShapeKind, vertices: Vec<Vertex>) -> Result<EntityBuilder, ShapeError> {
        if let Some(err) = Self::valid_vertex_number(&kind, vertices.len()) {
            return Err(err);
        }
        let points = match kind {
            ShapeKind::Triangle => vertices,
            ShapeKind::Rectangle => {
                vec![
                    vertices[0], vertices[1], vertices[2],
                    vertices[2], vertices[3], vertices[0]
                ]
            },
            ShapeKind::Circle(radius) => {
                let center = vertices[0];
                let mut points = Vec::with_capacity(360);
                let conversion_factor = std::f32::consts::PI / 180.0;
                points.push(Vertex::new(
                    center.position[0] + radius,
                    center.position[1],
                    0.0,
                    center.color.into()
                ));
                points.push(center);
                for i in 1..360 {
                    let theta = (i as f32) * conversion_factor;
                    points.push(Vertex::new(
                        center.position[0] + radius * theta.cos(),
                        center.position[1] + radius * theta.sin(),
                        0.0,
                        center.color.into()
                    ));
                    points.push(Vertex::new(
                        center.position[0] + radius * theta.cos(),
                        center.position[1] + radius * theta.sin(),
                        0.0,
                        center.color.into()
                    ));
                    points.push(center);
                }
                points.push(Vertex::new(
                    center.position[0] + radius,
                    center.position[1],
                    0.0,
                    center.color.into()
                ));
                points.into_iter().rev().collect()
            }
        };
        Ok(EntityBuilder::new(points))
    }

    pub fn build(self, gpu: &wgpu::Device, config: &wgpu::SurfaceConfiguration, width: u32, height: u32) -> Entity {
        Entity::new(gpu, config, width as f32, height as f32, self.vertices)
    }

}

#[derive(Debug, Clone, Copy)]
pub enum ShapeKind {
    Triangle,
    Rectangle,
    Circle(f32)
}

impl ShapeKind {

    pub fn requisite_points(&self) -> usize {
        match self {
            Self::Triangle => 3,
            Self::Rectangle => 4,
            Self::Circle(_) => 1
        }
    }

}

#[derive(Debug)]
pub enum ShapeError {
    VertexOverspecification(ShapeKind),
    VertexUnderspecification(ShapeKind)
}   

impl std::fmt::Display for ShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::VertexOverspecification(shape_kind) => match shape_kind {
                ShapeKind::Triangle => "A triangle requires only three vertices",
                ShapeKind::Rectangle => "A rectangle requires only four vertices",
                ShapeKind::Circle(_) => "A circle requires only one vertex for its center"
            },
            Self::VertexUnderspecification(shape_kind) => match shape_kind {
                ShapeKind::Triangle => "A triangle requires at least three vertices",
                ShapeKind::Rectangle => "A rectangle requires at least four vertices",
                ShapeKind::Circle(_) => "A circle requires a vertex for its center"
            }
        };
        write!(f, "{}", msg)
    }
}
