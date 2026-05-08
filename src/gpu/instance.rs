use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use crate::common::instance::MeshInstance;

/// A transformation matrix for a mesh that can be passed into the shaders.
#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct GpuInstance {
    #[format(R32G32B32A32_SFLOAT)]
    pub transform_col1: [f32; 4],
    #[format(R32G32B32A32_SFLOAT)]
    pub transform_col2: [f32; 4],
    #[format(R32G32B32A32_SFLOAT)]
    pub transform_col3: [f32; 4],
    #[format(R32G32B32A32_SFLOAT)]
    pub transform_col4: [f32; 4],
}

impl Into<GpuInstance> for MeshInstance {
    fn into(self) -> GpuInstance {
        let m = self.transform_matrix();
        GpuInstance {
            transform_col1: m.column(0).into(),
            transform_col2: m.column(1).into(),
            transform_col3: m.column(2).into(),
            transform_col4: m.column(3).into(),
        }
    }
}
