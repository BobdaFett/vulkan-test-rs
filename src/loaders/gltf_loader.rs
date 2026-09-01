use crate::loaders::mesh_loader::{MeshFileLoader, MeshInfo};
use anyhow::Result;
use std::path::Path;
use nalgebra::Matrix4;

pub struct GltfLoader;

impl GltfLoader {
    /// Traverses and loads all information from a GLTF/GLB node.
    pub fn traverse_node(
        node: &gltf::Node,
        parent_transform: &Matrix4<f32>,
        buffers: &[Vec<u8>],
        vertices: &mut Vec<[f32; 3]>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 3]>,
        indices: &mut Vec<u32>
    ) {

    }
}

impl MeshFileLoader for GltfLoader {
    fn load_mesh(&self, path: &Path) -> Result<MeshInfo> {
        let (document, buffers, _) = gltf::import(path)?;

        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::<u32>::new();

        let scenes = document.scenes();

        // We need to iterate through scenes and apply the transforms from each scene to their
        // corresponding vertices.
        println!("Parsing {} scenes from {:?}", scenes.len(), path);
        for scene in scenes {
            for node in scene.nodes() {
                let transform = node.transform().matrix();
                println!("Scene has transform: {:?}", transform);
            }
        }

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                // Read the primitives as vertices and indices,
                // and register a material for the primitive as well
                let vert_offset = vertices.len() as u32;

                let reader = primitive.reader(|buffer_index| Some(&buffers[buffer_index.index()]));

                if let Some(positions) = reader.read_positions() {
                    for position in positions {
                        vertices.push([position[0], position[2], -position[1]]);
                    }
                }

                if let Some(normals_iter) = reader.read_normals() {
                    for normal in normals_iter {
                        normals.push(normal);
                    }
                }

                if let Some(tex_coords) = reader.read_tex_coords(0) {
                    for uv in tex_coords.into_f32() {
                        uvs.push([uv[0], 1.0 - uv[1], 0.0]);
                    }
                }

                if let Some(indices_iter) = reader.read_indices() {
                    for index in indices_iter.into_u32() {
                        indices.push(index + vert_offset);
                    }
                }
            }
        }

        Ok(MeshInfo {
            vertices,
            indices,
            normals,
            uvs,
        })
    }
}
