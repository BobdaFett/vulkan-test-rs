use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(Debug, Clone, BufferContents, Vertex)]
#[repr(C)]
pub struct Vertex3 {
    #[format(R32G32B32_SFLOAT)]
    pub(crate) position: [f32; 3],
}

impl From<[f32; 3]> for Vertex3 {
    fn from(v: [f32; 3]) -> Self {
        Self {
            position: v.clone(),
        }
    }
}
