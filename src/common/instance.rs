use crate::common::scene::SceneInstance;
use nalgebra::{Matrix4, Vector3, Vector4};

/// Contains all information about an instance of a mesh, specifically its translation, rotation,
/// and scale. Note that the rotation is a [`Vector4`], and not a `Quaternion`.
///
/// This may be taken from a scene instance by calling the `Instance::from` method on the
/// corresponding [`SceneInstance`]. This is important, as the `Vector4` is not serializable.
///
/// Before passing this instance into the shaders, construct a transformation matrix by calling
/// [`MeshInstance::transform_matrix`].
pub struct MeshInstance {
    /// The ID of the mesh corresponding to this instance.
    pub mesh_id: String,
    /// The translation of the instance from the center of the standard space. It is possible that
    /// local translation will be required in the future.
    pub translation: Vector4<f32>,
    /// The rotation of the instance. Note that since this is a [`Vector4`], or Euler angle, it
    /// suffers from gimbal lock. If this becomes a serious issue, an [`nalgebra::UnitQuaternion`]
    /// should be used instead.
    pub rotation: Vector4<f32>,
    /// The scale of the instance, represented as the scaling in XYZ coordinates.
    pub scale: Vector3<f32>,
}

impl MeshInstance {
    /// Constructs a transformation matrix from the current translation, rotation, and scale.
    pub fn transform_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.translation.xyz()) *
            // There's a possibility that these can be altered. Sometimes they're multiplied in the
            // order ZYX. They are technically applied in the opposite order written here, so in
            // this case it's actually applying Z, then Y, then X. They *do* produce different results.
            Matrix4::from_axis_angle(&Vector3::x_axis(), self.rotation.x) *
            Matrix4::from_axis_angle(&Vector3::y_axis(), self.rotation.y) *
            Matrix4::from_axis_angle(&Vector3::z_axis(), self.rotation.z) *
            Matrix4::new_nonuniform_scaling(&self.scale)
    }

    /// Constructs a `MeshInstance` from the given [`SceneInstance`] and mesh ID.
    fn from_scene_instance(
        mesh_id: String,
        SceneInstance {
            translation,
            rotation,
            scale,
        }: SceneInstance,
    ) -> Self {
        Self {
            mesh_id,
            translation: Vector3::from(translation).to_homogeneous(),
            rotation: Vector3::from(rotation).to_homogeneous(),
            scale: Vector3::from(scale),
        }
    }
}
