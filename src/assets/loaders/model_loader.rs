use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use crate::assets::loaders::common::{MaterialLoadInfo, MeshLoadInfo, TextureLoadInfo};

/// A trait for loading a model from a given path.
pub trait LoadModel {
    /// Loads a model from the given path, returning a [`ModelLoadInformation`] if successful.
    fn load_model(&self, path: &Path, options: ModelLoadOptions) -> Result<ModelLoadInfo>;
}

/// The options applicable to loading a model.
#[derive(Default, Copy, Clone, Debug)]
pub struct ModelLoadOptions {
    /// Whether to load the full mesh.
    pub load_mesh: bool,
    /// Whether to load all materials from the file.
    pub load_materials: bool,
    /// Whether to load all textures from the file.
    pub load_textures: bool,
}

/// Contains the information about the mesh, materials, and textures that this model has loaded.
/// Each type of information will only be loaded if the caller indicates that they should be.
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

        Self {
            loaders,
        }
    }
}
