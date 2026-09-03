use std::path::Path;
use crate::loaders::material::mat_loader::{MaterialFileLoader, MaterialInfo};

pub struct GltfMaterialLoader;

impl MaterialFileLoader for GltfMaterialLoader {
    fn load_material(&self, path: &Path) -> anyhow::Result<MaterialInfo> {
        let (document, buffers, images) = gltf::import(path)?;

        // We just want to read through the information to see what kinds of materials are available.
        // I'm not sure how gltf defines materials, especially if they're applied to specific sets
        // of vertices.

        todo!()
    }
}
