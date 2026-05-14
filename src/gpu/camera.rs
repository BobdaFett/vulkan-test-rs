use vulkano::buffer::BufferContents;
use crate::common::camera::Camera;

#[derive(BufferContents)]
#[repr(C)]
pub struct CameraUniform {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub position: [f32; 3],
}

impl From<Camera> for CameraUniform {
    fn from(camera: Camera) -> Self {
        Self::from(&camera)
    }
}

impl From<&Camera> for CameraUniform {
    fn from(camera: &Camera) -> Self {
        let view = camera.get_view();
        let proj = camera.get_projection();

        Self {
            view: view.into(),
            proj: proj.into(),
            position: camera.position.into(),
        }
    }
}
