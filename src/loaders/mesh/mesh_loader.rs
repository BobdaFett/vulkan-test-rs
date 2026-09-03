use super::gltf_loader::*;
use crate::loaders::mesh::obj_loader::ObjLoader;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

pub struct MeshInfo {
    /// The mesh's vertices.
    pub vertices: Vec<[f32; 3]>,
    /// The mesh's vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// The mesh's vertex indices.
    pub indices: Vec<u32>,
    /// The texture coordinates. Note that these may also be used on 3D textures.
    pub uvs: Vec<[f32; 3]>,
}

pub trait MeshFileLoader {
    fn load_mesh(&self, path: &Path) -> Result<MeshInfo>;
}

#[derive(Error, Debug)]
pub enum MeshLoaderError {
    #[error("file extension is missing")]
    MissingExtension,
    #[error("extension {0} is not supported")]
    ExtensionNotSupported(String),
}

/// A mesh loader struct that handles multiple different file types. The list of supported and
/// planned file formats follows:
///  - `.gltf`
///  - `.obj`
pub struct MeshLoader {
    loaders: HashMap<String, Box<dyn MeshFileLoader>>,
}

impl MeshLoader {
    pub fn new() -> Self {
        let mut loaders = HashMap::new();
        loaders.insert(
            "gltf".into(),
            Box::new(GltfLoader) as Box<dyn MeshFileLoader>,
        );
        loaders.insert(
            "glb".into(),
            Box::new(GltfLoader) as Box<dyn MeshFileLoader>,
        );
        loaders.insert("obj".into(), Box::new(ObjLoader));

        Self { loaders }
    }

    /// Loads a mesh from the given path. Note that materials are not automatically loaded through
    /// this function, and must be manually loaded. Refer to the [`MeshLoader`] for more information
    /// on what file types are supported.
    pub fn load_mesh<P: AsRef<Path>>(&self, path: &P) -> Result<MeshInfo> {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(MeshLoaderError::MissingExtension)?;

        println!("Found extension {}, continuing with loading", ext);

        self.loaders
            .get(ext)
            .ok_or(MeshLoaderError::ExtensionNotSupported(ext.to_string()))?
            .load_mesh(path.as_ref())
    }
}

#[cfg(test)]
pub mod test_load_meshes {
    use super::*;

    fn load_gltf(loader: &MeshLoader) -> Result<MeshInfo> {
        loader.load_mesh(&Path::new("../../../meshes/laat_gltf/scene.gltf"))
    }

    fn load_obj(loader: &MeshLoader) -> Result<MeshInfo> {
        loader.load_mesh(&Path::new("../../../meshes/laat_gunship/scene.obj"))
    }

    #[test]
    fn compare_obj_gltf() -> Result<()> {
        // Create the mesh loader
        let loader = MeshLoader::new();

        let obj = load_obj(&loader)?;
        let gltf = load_gltf(&loader)?;

        // Compare mesh information between the two MeshInfo structs.
        obj.vertices.iter()
            .zip(gltf.vertices.iter())
            .for_each(|(&a, &b)| assert_eq!(a, b));
        obj.indices.iter()
            .zip(gltf.indices.iter())
            .for_each(|(&a, &b)| assert_eq!(a, b));
        obj.normals.iter()
            .zip(gltf.normals.iter())
            .for_each(|(&a, &b)| assert_eq!(a, b));

        Ok(())
    }
}
