
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Point {
    position: [f32; 3],
    color: [f32; 3],
}

impl Point {
    pub fn new(x: f32, y: f32, color: [f32; 3]) -> Self {
        Self {
            position: [x, y, 0.0],
            color,
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Point>() as wgpu::BufferAddress,
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

pub struct Entity {
    pub points: Vec<Point>,
    pub point_order: Vec<u32>,
}

impl Entity {

    fn point_order_from_points(points: &[Point]) -> Vec<u32> {
        let mut order: Vec<u32> = vec![];
        for (i, _) in points.iter().enumerate() {
            order.push(i as u32);
        }
        order
    }

    pub fn from_points(points: Vec<Point>) -> Self {
        let order = Entity::point_order_from_points(&points);
        Self {
            points,
            point_order: order,
        }
    }
}