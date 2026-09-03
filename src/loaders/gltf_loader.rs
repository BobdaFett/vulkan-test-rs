use crate::loaders::mesh_loader::{MeshFileLoader, MeshInfo};
use anyhow::Result;
use std::path::Path;
use gltf::buffer::Data;
use nalgebra::{Matrix4, Vector4};

pub struct GltfLoader;

impl GltfLoader {
    /// Traverses and loads all information from a GLTF/GLB node.
    pub fn traverse_node(
        node: &gltf::Node,
        parent_transform: &Matrix4<f32>,
        buffers: &Vec<Data>,
        vertices: &mut Vec<[f32; 3]>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 3]>,
        indices: &mut Vec<u32>
    ) {
        let transform = node.transform().matrix()
            .iter()
            .flatten()
            .cloned()
            .collect::<Vec<f32>>();
        let matrix = Matrix4::from_column_slice(transform.as_slice());
        let full_transform = parent_transform * matrix;

        if let Some(mesh) = node.mesh() {
            // Read the mesh and apply the current transformation to every vertex
            for primitive in mesh.primitives() {
                let vert_offset = vertices.len() as u32;

                let reader = primitive.reader(|buffer_index| Some(&buffers[buffer_index.index()]));

                if let Some(positions) = reader.read_positions() {
                    for position in positions {
                        // Transform into a Vector4, apply transform, push as 3D slice
                        let pos_vec = Vector4::new(position[0], position[1], position[2], 1.0);
                        let transformed = full_transform * pos_vec;
                        vertices.push([transformed.x, transformed.y, transformed.z]);
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

        // Process all child nodes as well
        for child in node.children() {
            Self::traverse_node(
                &child,
                &full_transform,
                &buffers,
                vertices,
                normals,
                uvs,
                indices
            );
        }
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
                // Read through the nodes and get all vertex information.
                // Even if there are multiple scenes, we're treating the whole file as a single mesh.
                Self::traverse_node(
                    &node,
                    &Matrix4::identity(),
                    &buffers,
                    &mut vertices,
                    &mut normals,
                    &mut uvs,
                    &mut indices
                );
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
