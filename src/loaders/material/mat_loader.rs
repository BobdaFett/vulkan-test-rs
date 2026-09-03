use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use anyhow::Result;
use crate::loaders::material::gltf_mat_loader::GltfMaterialLoader;

#[derive(Error, Debug)]
pub enum MaterialLoaderError {
    #[error("file extension is missing")]
    MissingExtension,
    #[error("extension {0} is not supported")]
    ExtensionNotSupported(String)
}

/// A material loader struct that handles multiple different file types.
/// List of supported and planned file formats:
///  - `.gltf`
///  - `.mtl` (planned, not implemented)
pub struct MaterialLoader {
    loaders: HashMap<String, Box<dyn MaterialFileLoader>>,
}

impl MaterialLoader {
    /// Creates a new `MaterialLoader` struct.
    pub fn new() -> Self {
        let mut loaders = HashMap::new();
        loaders.insert("gltf".into(), Box::new(GltfMaterialLoader) as Box<dyn MaterialFileLoader>);

        Self { loaders }
    }

    /// Loads a material from the given path. Note that some materials are loaded through their
    /// mesh's file (GLTF embeds materials, however OBJ files have a separate .mtl file). These
    /// quirks must be handled differently. Refer to the [`MaterialLoader`] for more information
    /// on supported material files and which files to target.
    pub fn load_material<P: AsRef<Path>>(&mut self, path: P) -> Result<MaterialInfo> {
        let path = path.as_ref();
        let file_ext = path.extension().and_then(|ext| ext.to_str())
            .ok_or(MaterialLoaderError::MissingExtension)?;

        self.loaders.get(file_ext)
            .ok_or(MaterialLoaderError::ExtensionNotSupported(file_ext.to_string()))?
            .load_material(path)
    }
}

/// Contains all information required for the application to load a new material.
pub struct MaterialInfo {
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

/// A trait that must be implemented on all material loaders. Ensures consistent material
/// import/load information.
pub trait MaterialFileLoader {
    fn load_material(&self, path: &Path) -> anyhow::Result<MaterialInfo>;
}
