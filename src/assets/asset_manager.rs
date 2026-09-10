use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use vulkano::memory::allocator::MemoryAllocator;
use crate::assets::loaders::material::mat_loader::MaterialLoader;
use crate::assets::loaders::mesh_loader::MeshLoader;
use crate::common::material::MaterialRegistry;
use crate::common::mesh::{MeshHandle, MeshRegistry};

/// A manager that controls all assets used for display. It handles loading all meshes, materials,
/// and textures for now.
pub struct AssetManager {
    /// The asset manager's [`MeshRegistry`].
    mesh_registry: MeshRegistry,
    /// The asset manager's [`MaterialRegistry`].
    material_registry: MaterialRegistry,

    // Loaders
    mesh_loader: MeshLoader,
    material_loader: MaterialLoader,
}

impl AssetManager {
    /// Creates a new `AssetManager` instance.
    pub fn new(allocator: Arc<dyn MemoryAllocator>) -> Self {
        let mesh_registry = MeshRegistry::new(allocator.clone());
        let material_registry = MaterialRegistry::new();
        Self {
            mesh_registry,
            material_registry,
            mesh_loader: MeshLoader::new(),
            material_loader: MaterialLoader::new(),
        }
    }

    /// Loads a full model from its file path. This loads the model's mesh, textures, and material
    /// definitions, registers them with the `AssetLoader`'s registries, and prepares them for
    /// display. Returns a blank `Result` indicating success or failure.
    ///
    /// This method will attempt to auto-generate IDs for materials and textures if they do not have
    /// any preconfigured, using the ID of the mesh and the type of resource being stored.
    pub fn load_model<P: AsRef<Path>>(&mut self, _mesh_id: String, path: P) -> Result<()> {
        // Load and register all materials first - submeshes will reference these materials.
        let _materials = self.material_loader.load_materials(&path)?;

        // Load the mesh. Each of `mesh.submeshes` carries a `material_index` local to this file,
        // which should be resolved against `materials` (e.g. via a generated
        // "{mesh_id}_mat_{index}" id) once there's a way to register it.
        let _mesh = self.mesh_loader.load_mesh(&path)?;

        // `MeshRegistry` only knows how to allocate its shared vertex/index buffers in one batch
        // via `load_scene`; there's no way yet to register a single mesh's `MeshInfo` into
        // already-allocated buffers (or grow them), so this can't build a `MeshHandle` yet.
        todo!("MeshRegistry needs support for registering a single mesh after its buffers are already allocated")
    }

    /// Loads all information from a scene, returning an empty `Result` indicating success or failure.
    pub fn load_scene<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Ok(())
    }
}
