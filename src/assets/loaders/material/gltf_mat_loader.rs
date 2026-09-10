use crate::assets::loaders::material::mat_loader::{MaterialFileLoader, MaterialInfo};
use std::path::Path;

pub struct GltfMaterialLoader;

impl MaterialFileLoader for GltfMaterialLoader {
    fn load_materials(&self, path: &Path) -> anyhow::Result<Vec<MaterialInfo>> {
        let (document, buffers, images) = gltf::import(path)?;

        for material in document.materials() {

        }

        for scene in document.scenes() {
            for node in scene.nodes() {
                
            }
        }

        // We just want to read through the information to see what kinds of materials are available.
        // I'm not sure how gltf defines materials, especially if they're applied to specific sets
        // of vertices.

        todo!()
    }
}
