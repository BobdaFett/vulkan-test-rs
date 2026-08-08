use std::path::Path;
use anyhow::Result;
use crate::loaders::mesh_loader::{MeshFileLoader, MeshInfo};

pub struct GltfLoader;

impl MeshFileLoader for GltfLoader {
    fn load_mesh(&self, path: &Path) -> Result<MeshInfo> {
        todo!()
    }
}