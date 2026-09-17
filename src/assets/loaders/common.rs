pub struct MeshLoadInfo {
    /// The mesh's vertices.
    pub vertices: Vec<[f32; 3]>,
    /// The mesh's vertex normals.
    pub normals: Vec<[f32; 3]>,
    /// The mesh's vertex indices.
    pub indices: Vec<u32>,
    /// The texture coordinates. Note that these may also be used on 3D textures.
    pub uvs: Vec<[f32; 3]>,
    /// The contiguous ranges of `indices` that should be drawn with a single material, in the
    /// order they were encountered in the source file.
    pub submeshes: Vec<Submesh>,
}

/// A contiguous range within a [`MeshInfo`]'s `indices`, along with the material it should be
/// drawn with. Ranges are local to a single loaded file - a [`crate::common::mesh::MeshRegistry`]
/// rewrites these into offsets within its own shared index buffer.
#[derive(Debug, Clone)]
pub struct Submesh {
    /// The index of the material this submesh uses, as defined by the source file (e.g. a glTF
    /// material index). `None` if the file did not assign a material to this range.
    pub material_index: Option<usize>,
    /// The offset of this submesh's first index within `MeshInfo::indices`.
    pub index_start: usize,
    /// The number of indices that make up this submesh.
    pub index_count: usize,
}


/// Contains all information required for the application to load a new material.
pub struct MaterialLoadInfo {
    /// The ID of this material
    pub name: String,
    /// The material uniforms. These should be defined on every imported material.
    pub uniforms: MaterialUniforms,
    /// The ID of the base color map, if any.
    pub base_color_map: Option<String>,
    /// The ID of the roughness map, if any. This is ignored for now.
    pub roughness_map: Option<String>,
    /// The ID of the normal map, if any. This is ignored for now.
    pub normal_map: Option<String>,
}

/// A struct composed of the material uniforms. These uniforms will typically scale the material
/// texture maps if there are any. Refer to [`MaterialInfo`] for more information.
pub struct MaterialUniforms {
    /// The base color of the material. This scales the base color texture, if any.
    /// This value must be specified in normalized RGBA format (all values between 0 and 1).
    pub base_color: [f32; 4],
    /// The metalness of the material. This scales the metalness texture, if any. This value should
    /// be specified as a normalized float between 0 and 1, based on an 8-bit number space.
    ///
    /// This is ignored until the shaders are updated.
    pub metalness_factor: f32,
    /// The roughness of the material. This scales the roughness texture, if any. This value should
    /// be specified as a normalized float between 0 and 1, based on an 8-bit number space.
    ///
    /// This is ignored until the shaders are updated.
    pub roughness_factor: f32,
}

/// A struct containing all information about a loaded texture.
pub struct TextureLoadInfo {
}
