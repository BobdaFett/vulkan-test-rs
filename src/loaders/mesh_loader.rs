use super::gltf_loader::*;
use crate::loaders::obj_loader::ObjLoader;
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

pub struct MeshLoader {
    loaders: HashMap<String, Box<dyn MeshFileLoader>>,
}

#[derive(Error, Debug)]
pub enum MeshLoaderError {
    #[error("file extension is missing")]
    MissingExtension,
    #[error("extension {0} is not supported")]
    ExtensionNotSupported(String),
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

    /// Loads a mesh and returns a `Result` containing the information.
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
