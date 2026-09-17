use vulkano::buffer::BufferContents;

/// Mirrors the `MaterialPushConstants` block in `triangle_frag.glsl`. Pushed once per submesh
/// draw call, since each submesh may use a different material.
#[derive(BufferContents, Debug, Clone, Copy)]
#[repr(C)]
pub struct MaterialPushConstants {
    pub base_color: [f32; 4],
}
