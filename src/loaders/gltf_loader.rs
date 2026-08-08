use std::path::Path;
use anyhow::{anyhow, Result};
use crate::loaders::mesh_loader::{MeshFileLoader, MeshInfo};

pub struct GltfLoader;

impl MeshFileLoader for GltfLoader {
    fn load_mesh(&self, path: &Path) -> Result<MeshInfo> {
        let scenes = easy_gltf::load(path)
            .map_err(|e| anyhow!("could not load gltf file: {e}"))?;

        println!("Found {} scene(s) in gltf file", scenes.len());

        let scene = scenes.first()
            .ok_or(anyhow!("no scenes in gltf file"))?;

        println!("Found {} model(s) in gltf file", scene.models.len());

        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::<u32>::new();
        for model in &scene.models {
            // Indices must be offset by the number of vertices that are already registered.
            let idx_offset = vertices.len() as u32;

            // Put all vertex info together for a MeshInfo struct
            for vertex in model.vertices() {
                vertices.push(*vertex.position.as_ref());
                normals.push(*vertex.normal.as_ref());
                uvs.push([vertex.tex_coords.x, vertex.tex_coords.y, 0.0]);
            }
            model.indices()
                .expect("failed to find indices")
                .iter()
                .map(|i| i + idx_offset)
                .for_each(|i| indices.push(i));
        }

        Ok(MeshInfo {
            vertices,
            indices,
            normals,
            uvs
        })
    }
}