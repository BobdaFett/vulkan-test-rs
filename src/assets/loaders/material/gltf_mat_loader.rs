use crate::assets::loaders::material::mat_loader::{MaterialFileLoader, MaterialInfo, MaterialUniforms};
use gltf::image::Source;
use gltf::texture::Texture;
use std::path::Path;

pub struct GltfMaterialLoader;

impl GltfMaterialLoader {
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
}

impl MaterialFileLoader for GltfMaterialLoader {
    fn load_materials(&self, path: &Path) -> anyhow::Result<Vec<MaterialInfo>> {
        let (document, _buffers, _images) = gltf::import(path)?;

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

                MaterialInfo {
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
}
