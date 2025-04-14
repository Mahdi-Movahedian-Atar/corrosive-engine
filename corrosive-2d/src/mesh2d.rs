#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex2D {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}
