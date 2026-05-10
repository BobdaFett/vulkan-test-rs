use std::collections::HashMap;
use crate::common::scene::{Scene, SceneInstance};
use nalgebra::{Matrix4, Vector3, Vector4};

/// Contains all information about an instance of a mesh, specifically its translation, rotation,
/// and scale. Note that the rotation is a [`Vector4`], and not a `Quaternion`.
///
/// This may be taken from a scene instance by calling the `Instance::from` method on the
/// corresponding [`SceneInstance`]. This is important, as the `Vector4` is not serializable.
///
/// Before passing this instance into the shaders, construct a transformation matrix by calling
/// [`MeshInstance::transform_matrix`].
#[derive(Debug, Clone)]
pub struct MeshInstance {
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
}

impl From<SceneInstance> for MeshInstance {
    fn from(instance: SceneInstance) -> Self {
        Self::from(&instance)
    }
}

impl From<&SceneInstance> for MeshInstance {
    fn from(SceneInstance { translation, rotation, scale }: &SceneInstance) -> Self {
        Self {
            translation: Vector3::from(*translation).to_homogeneous(),
            rotation: Vector3::from(*rotation).to_homogeneous(),
            scale: Vector3::from(*scale),
        }
    }
}

/// A registry containing a series of `MeshInstance` structs linked to a mesh ID.
pub struct InstanceRegistry {
    instances: HashMap<String, Vec<MeshInstance>>,
}

impl InstanceRegistry {
    pub fn from_scene(scene: &Scene) -> Self {
        // Read information about the instances from the scene.
        let instances: HashMap<String, Vec<MeshInstance>> = scene.instances.iter()
            .fold(HashMap::new(), |mut map, (mesh_id, instance)| {
                map.entry(mesh_id.clone())
                    .and_modify(|v| v.push(instance.into()))
                    .or_insert(vec![instance.into()]);
                map
            });

        Self {
            instances
        }
    }

    pub fn get_instances_for(&self, mesh_id: &String) -> Vec<MeshInstance> {
        // We can't move these instances because we'll want them again later.
        self.instances.get(mesh_id).cloned().unwrap_or_default()
    }
}
