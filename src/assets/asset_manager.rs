use crate::common::material::{Material, MaterialRegistry};
use crate::common::mesh::{MeshHandle, MeshRegistry};
use crate::common::scene::Scene;
use crate::common::texture::TextureRegistry;
use crate::gpu::vertex3::Vertex3;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use vulkano::buffer::Subbuffer;
use vulkano::memory::allocator::MemoryAllocator;
use crate::assets::loaders::model_loader::{ModelLoadOptions, ModelLoader};

/// A manager that controls all assets used for display. It handles loading all meshes, materials,
/// and textures for now.
///
/// Callers outside this module should go through `AssetManager` rather than reaching into a
/// [`MeshRegistry`]/[`MaterialRegistry`] directly - it's the thing that will keep meshes and their
/// materials consistent with each other once material loading is implemented.
pub struct AssetManager {
    /// The memory allocator used to (re)allocate the mesh registry's buffers.
    allocator: Arc<dyn MemoryAllocator>,
    /// The asset manager's [`MeshRegistry`].
    mesh_registry: MeshRegistry,
    /// The asset manager's [`MaterialRegistry`].
    material_registry: MaterialRegistry,
    /// The asset manager's [`TextureRegistry`].
    texture_registry: TextureRegistry,
}

impl AssetManager {
    /// Creates a new `AssetManager` instance.
    pub fn new(allocator: Arc<dyn MemoryAllocator>) -> Self {
        let mesh_registry = MeshRegistry::new(allocator.clone());
        let material_registry = MaterialRegistry::new();
        Self {
            allocator,
            mesh_registry,
            material_registry,
            texture_registry: TextureRegistry::new(),
        }
    }

    /// Loads a full model from its file path. This loads the model's mesh, textures, and material
    /// definitions, registers them with the `AssetLoader`'s registries, and prepares them for
    /// display. Returns a blank `Result` indicating success or failure.
    ///
    /// This method will attempt to auto-generate IDs for materials and textures if they do not have
    /// any preconfigured, using the ID of the mesh and the type of resource being stored.
    pub fn load_model<P: AsRef<Path>>(&mut self, mesh_id: &String, path: P) -> Result<()> {
        // Submeshes reference materials as "{mesh_id}_mat_{index}" (see
        // `MeshRegistry::append_mesh`), matching the index materials are registered under below.
        // Not every mesh format has a material loader yet (e.g. `.obj`), so a failure here just
        // means no materials get registered for this mesh rather than failing the whole load.
        let path = path.as_ref();
        let mut loader = ModelLoader::new();
        let model_info = loader.load_model(path, ModelLoadOptions::default())?;
        
        match model_info.materials {
            Some(materials) => {
                for (index, info) in materials.into_iter().enumerate() {
                    let material_id = format!("{mesh_id}_mat_{index}");
                    self.material_registry
                        .register(material_id.clone(), info.into());
                    println!("Loaded material {material_id}");
                }
            }
            None => println!("No materials loaded for mesh \"{mesh_id}\""),
        }

        let mesh = model_info.mesh
            .expect("Mesh was not loaded");
        self.mesh_registry.register_mesh(mesh_id.clone(), mesh);

        Ok(())
    }

    /// Loads every mesh referenced by a [`Scene`] and allocates the buffers needed to render them.
    pub fn load_scene(&mut self, scene: &Scene) -> Result<()> {
        for (id, path) in &scene.mesh_paths {
            self.load_model(id, path)
                .expect("Couldn't load mesh file");
        }

        Ok(())
    }

    /// Returns the shared vertex buffer backing every mesh currently registered.
    pub fn vertex_buffer(&self) -> Subbuffer<[Vertex3]> {
        self.mesh_registry
            .vertex_buffer
            .as_ref()
            .expect("Vertex buffer not allocated")
            .as_ref()
            .clone()
    }

    /// Returns the shared index buffer backing every mesh currently registered.
    pub fn index_buffer(&self) -> Subbuffer<[u32]> {
        self.mesh_registry
            .index_buffer
            .as_ref()
            .expect("Index buffer not allocated")
            .as_ref()
            .clone()
    }

    /// Looks up a registered mesh by its ID.
    pub fn get_mesh(&self, mesh_id: &String) -> Option<&MeshHandle> {
        self.mesh_registry.get(mesh_id)
    }

    /// Returns every currently registered mesh, keyed by ID.
    pub fn meshes(&self) -> &HashMap<String, MeshHandle> {
        &self.mesh_registry.meshes
    }

    /// Looks up a registered material by its ID (e.g. a [`crate::common::mesh::SubmeshHandle`]'s
    /// `material_id`).
    pub fn get_material(&self, material_id: &String) -> Option<&Material> {
        self.material_registry.get(material_id)
    }
}
