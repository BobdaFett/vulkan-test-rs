use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(Debug, Clone, BufferContents, Vertex)]
#[repr(C)]
pub struct Vertex3 {
    #[format(R32G32B32_SFLOAT)]
    pub(crate) position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub(crate) normal: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub(crate) uv: [f32; 3],
}

impl Vertex3 {
    pub fn new(
        position: [f32; 3],
        normal: [f32; 3],
        uv: [f32; 3],
    ) -> Self {
        Self {
            position,
            normal,
            uv
        }
    }
}
