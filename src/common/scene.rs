use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

/// A scene, which holds all information required to render a world.
///
/// This includes all mesh information, and all the information that is required to place
/// instances of these meshes in the world.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Scene {
    /// The name of the scene.
    pub name: String,
    /// The collection of paths to the meshes, relative to the current scene file.
    /// This collection is keyed on the ID of the mesh, which can be used later on to place
    /// the meshes in the scenes with specific instances. This ID will also be used to register
    /// the meshes with the application's registries, which will allow the program to associate
    /// specific behavior with the mesh.
    pub mesh_paths: HashMap<String, String>,
    /// A collection of instances that are keyed by the mesh's IDs. Each instance can be used to
    /// place a [`Mesh`] in the rendered scene.
    pub instances: HashMap<String, SceneInstance>,
}

impl Scene {
    /// Converts this `Scene` into its byte representation.
    pub fn bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(self).expect("Couldn't serialize scene")
    }

    /// Reads a `scene` from the given path. Automatically resolves all paths for the meshes that
    /// are referenced in the struct.
    ///
    /// Reading a scene file allows for all meshes to be loaded at once, which therefore allows
    /// the program to allocate pipeline buffers just once during the load of this. It would likely
    /// be reasonable to create a `MeshRegistry` and `InstanceRegistry` at the same time, with
    /// respect to a single loaded `Scene`.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let mut file = std::fs::File::open(path).unwrap();
        let mut contents = Vec::new();
        let _ = file.read_to_end(&mut contents).unwrap();

        let scene: Self =
            postcard::from_bytes(contents.as_slice()).expect("Couldn't deserialize scene");

        Self::resolve_paths(path, scene)
    }

    pub fn from_file_json<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        let file = std::fs::File::open(path).unwrap();
        let scene: Self = serde_json::from_reader(file).unwrap();

        Self::resolve_paths(path, scene)
    }

    fn resolve_paths(scene_path: &Path, mut scene: Self) -> Self {
        let base = scene_path.parent().unwrap_or(&Path::new("."));
        scene.mesh_paths = scene
            .mesh_paths
            .into_iter()
            .map(|(id, mesh_path)| {
                let res = base.join(mesh_path);
                let str = res.to_str().unwrap_or_default();

                (id, str.to_string())
            })
            .collect::<HashMap<String, String>>();

        scene
    }
}

/// Contains all information about an instance in the scene, specifically the name of the mesh it
/// should use and the translation, rotation, and scale of it.
///
/// After deserialization, this struct should be converted into an [`Instance`], which contains
/// defined vectors that can be used to work with the object before sending it to the shaders.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct SceneInstance {
    pub translation: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

#[cfg(test)]
pub mod create_files {
    use std::error::Error;
    use std::io::Write;
    pub use super::*;

    #[test]
    pub fn create_test_scene() -> Result<(), Box<dyn Error>> {
        let scene = Scene {
            name: "Testing Scene".to_string(),
            mesh_paths: HashMap::from([
                ("venator".to_string(), "Venator.obj".to_string())
            ]),
            instances: HashMap::from([
                ("venator".to_string(), SceneInstance {
                    translation: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0]
                })
            ])
        };

        let json = serde_json::to_string_pretty(&scene)?;

        let mut file = std::fs::File::create("test_scene.scene")?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }
}
