use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use crate::assets::loaders::common::{MaterialLoadInfo, MeshLoadInfo, TextureLoadInfo};
use crate::assets::loaders::gltf_loader::GltfLoader;

/// A trait for loading a model from a given path.
pub trait LoadModel {
    /// Loads a model from the given path, returning a [`ModelLoadInformation`] if successful.
    fn load_model(&self, path: &Path, options: ModelLoadOptions) -> Result<ModelLoadInfo>;
}

/// The options applicable to loading a model.
#[derive(Copy, Clone, Debug)]
pub struct ModelLoadOptions {
    /// Whether to load the full mesh.
    pub load_mesh: bool,
    /// Whether to load all materials from the file.
    pub load_materials: bool,
    /// Whether to load all textures from the file.
    pub load_textures: bool,
}

impl Default for ModelLoadOptions {
    fn default() -> Self {
        Self {
            load_mesh: true,
            load_materials: true,
            load_textures: true,
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum ModelLoadError {
    #[error("Extension was not found")]
    ExtensionNotFound,
    #[error("Extension {0} is not supported")]
    ExtensionNotSupported(String),
}

/// Contains the information about the mesh, materials, and textures that this model has loaded.
/// Each type of information will only be loaded if the caller indicates that they should be.
#[derive(Default)]
pub struct ModelLoadInfo {
    /// The loaded mesh, if applicable.
    pub mesh: Option<MeshLoadInfo>,
    /// The list of loaded materials, if applicable.
    pub materials: Option<Vec<MaterialLoadInfo>>,
    /// The list of loaded textures, if applicable.
    pub textures: Option<Vec<TextureLoadInfo>>,
}

/// A model loader struct that handles loading model information for various file types.
pub struct ModelLoader {
    /// The list of loaders, keyed by file type extension.
    loaders: HashMap<String, Box<dyn LoadModel>>
}

impl ModelLoader {
    /// Creates a new `ModelLoader` instance.
    pub fn new() -> Self {
        let mut loaders =  HashMap::new();
        loaders.insert("gltf".into(), Box::new(GltfLoader) as Box<dyn LoadModel>);
        loaders.insert("glb".into(), Box::new(GltfLoader));

        Self {
            loaders,
        }
    }

    /// Loads a model from the given path. The available options are indicated by the [`ModelLoadOptions`]
    /// struct.
    pub fn load_model<P: AsRef<Path>>(&mut self, path: P, options: ModelLoadOptions) -> Result<ModelLoadInfo> {
        // Determine which loader type to use.
        let path = path.as_ref();
        let ext = path.extension()
            .ok_or(ModelLoadError::ExtensionNotFound)?
            .to_str().unwrap();

        self.loaders.get(ext)
            .ok_or(ModelLoadError::ExtensionNotSupported(ext.into()))?
            .load_model(path, options)
    }
}
