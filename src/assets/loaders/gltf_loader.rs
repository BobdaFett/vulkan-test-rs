use anyhow::Result;
use std::path::Path;
use gltf::buffer::Data as BufferData;
use gltf::image::Data as ImageData;
use gltf::image::Source;
use gltf::{Document, Texture};
use nalgebra::{Matrix4, Vector4};
use crate::assets::loaders::common::{MaterialLoadInfo, MaterialUniforms, MeshLoadInfo, Submesh, TextureLoadInfo};
use crate::assets::loaders::model_loader::{LoadModel, ModelLoadInfo, ModelLoadOptions};

pub struct GltfLoader;

impl GltfLoader {
    /// Returns a stable identifier for a texture's image, meant to be resolved by a texture
    /// loader later. External images are identified by their URI; images embedded in the file
    /// itself (e.g. in a `.glb`) have no URI, so they're identified by their image index instead.
    fn texture_id(texture: &Texture) -> String {
        let image = texture.source();
        match image.source() {
            Source::Uri { uri, .. } => uri.to_string(),
            Source::View { .. } => format!("embedded_image_{}", image.index()),
        }
    }

    /// Loads a mesh from the given [`Document`] and vector of [`Data`], both specific to the glTF
    /// file format.
    fn load_mesh(&self, document: &Document, buffers: &Vec<BufferData>) -> Result<MeshLoadInfo> {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::<u32>::new();
        let mut submeshes = Vec::new();

        let scenes = document.scenes();

        // We need to iterate through scenes and apply the transforms from each scene to their
        // corresponding vertices.
        for scene in scenes {
            for node in scene.nodes() {
                // Read through the nodes and get all vertex information.
                // Even if there are multiple scenes, we're treating the whole file as a single mesh.
                Self::mesh_traverse_node(
                    &node,
                    &Matrix4::identity(),
                    &buffers,
                    &mut vertices,
                    &mut normals,
                    &mut uvs,
                    &mut indices,
                    &mut submeshes,
                );
            }
        }

        let info = MeshLoadInfo {
            vertices,
            indices,
            normals,
            uvs,
            submeshes,
        };

        Ok(info)
    }

    /// Traverses and loads all mesh information from a glTF/glB node.
    pub fn mesh_traverse_node(
        node: &gltf::Node,
        parent_transform: &Matrix4<f32>,
        buffers: &Vec<BufferData>,
        vertices: &mut Vec<[f32; 3]>,
        normals: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 3]>,
        indices: &mut Vec<u32>,
        submeshes: &mut Vec<Submesh>,
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
                let index_start = indices.len();

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

                submeshes.push(Submesh {
                    material_index: primitive.material().index(),
                    index_start,
                    index_count: indices.len() - index_start,
                });
            }
        }

        // Process all child nodes as well
        for child in node.children() {
            Self::mesh_traverse_node(
                &child,
                &full_transform,
                &buffers,
                vertices,
                normals,
                uvs,
                indices,
                submeshes,
            );
        }
    }

    /// Loads all materials from the given [`Document`] and [`Data`] buffers. These are glTF/glB
    /// specific.
    ///
    /// Note that this does not load any textures that are used with the materials, so this must
    /// be called in tandem with [`GltfLoader::load_textures`].
    fn load_materials(&self, document: &Document, buffers: &Vec<BufferData>) -> anyhow::Result<Vec<MaterialLoadInfo>> {
        // Collected in document order, so a material's position here matches the
        // `primitive.material().index()` that mesh loading recorded for its submeshes.
        Ok(document
            .materials()
            .enumerate()
            .map(|(index, material)| {
                let pbr = material.pbr_metallic_roughness();

                let name = material
                    .name()
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("material_{index}"));

                let base_color_map = pbr
                    .base_color_texture()
                    .map(|info| Self::texture_id(&info.texture()));
                let roughness_map = pbr
                    .metallic_roughness_texture()
                    .map(|info| Self::texture_id(&info.texture()));
                let normal_map = material
                    .normal_texture()
                    .map(|info| Self::texture_id(&info.texture()));

                MaterialLoadInfo {
                    name,
                    uniforms: MaterialUniforms {
                        base_color: pbr.base_color_factor(),
                        metalness_factor: pbr.metallic_factor(),
                        roughness_factor: pbr.roughness_factor(),
                    },
                    base_color_map,
                    roughness_map,
                    normal_map,
                }
            })
            .collect())
    }

    /// Loads all textures from the given [`Document`] and [`Data`] buffers.
    ///
    /// Note that loading this does not load any materials that may use the textures, so this must
    /// be called in tandem with [`GltfLoader::load_materials`].
    fn load_textures(&self, document: &Document, buffers: &Vec<BufferData>, images: &Vec<ImageData>) -> Result<Vec<TextureLoadInfo>> {
        todo!()
    }
}

impl LoadModel for GltfLoader {
    fn load_model(&self, path: &Path, options: ModelLoadOptions) -> Result<ModelLoadInfo> {
        let mut info = ModelLoadInfo::default();

        // First try to parse the file at the given path and get the information from it, then
        // continue with processing.
        let (document, buffers, images) = gltf::import(path)?;

        if options.load_mesh {
            // Load the mesh.
            let mesh_info = self.load_mesh(&document, &buffers)?;
            info.mesh = Some(mesh_info);
        }

        if options.load_materials {
            // Load the materials.
            let material_info = self.load_materials(&document, &buffers)?;
            info.materials = Some(material_info);
        }

        // TODO Textures aren't currently supported.
        // if options.load_textures {
        //     // Load the textures.
        //     let texture_info = self.load_textures(&document, &buffers, &images)?;
        //     info.textures = Some(texture_info)
        // }

        Ok(info)
    }
}
