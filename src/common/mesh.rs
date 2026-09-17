use crate::assets::loaders::mesh_loader::MeshLoadInfo;
use crate::gpu::vertex3::Vertex3;
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};

/// This struct is slightly misleading - it doesn't actually contain the mesh information, but
/// the locations of the mesh's information in the overall application's vertex buffer, index buffer,
/// and other such structures.
///
/// That means that this `MeshHandle` struct is actually just an entry within that registry. They
/// should never be edited or created manually.
#[derive(Debug)]
pub struct MeshHandle {
    pub id: String,
    pub vertex_loc: usize,
    pub vertex_count: usize,
    pub index_loc: usize,
    pub index_count: usize,
    /// The submeshes that make up this mesh, in the order they should be drawn. Each one covers
    /// a contiguous range of the registry's shared index buffer and (once material loading is
    /// wired up) should be drawn with its own material bound.
    pub submeshes: Vec<SubmeshHandle>,
}

impl MeshHandle {}

/// A submesh's location within a [`MeshRegistry`]'s shared index buffer, along with a reference
/// to the material it was assigned in its source file.
///
/// Unlike [`crate::assets::loaders::mesh_loader::Submesh`], `index_loc` here is an absolute
/// offset into the registry's shared index buffer rather than an offset local to one loaded file.
#[derive(Debug, Clone)]
pub struct SubmeshHandle {
    /// The id this submesh's material should be looked up under in a
    /// [`crate::common::material::MaterialRegistry`], if the source file assigned one. Follows
    /// the `"{mesh_id}_mat_{index}"` convention - nothing requires a material actually be
    /// registered under this id yet, until material loading is wired up.
    pub material_id: Option<String>,
    pub index_loc: usize,
    pub index_count: usize,
}

type VertexBuffer = Arc<Subbuffer<[Vertex3]>>;
type IndexBuffer = Arc<Subbuffer<[u32]>>;

/// # Mesh Registry
/// A wrapper around a series of [`MeshHandle`] structs, keyed by a unique ID, and the buffers that are
/// used by the rendering pipeline.
#[derive(Debug)]
pub struct MeshRegistry {
    /// The map of mesh IDs and their display handles.
    pub meshes: HashMap<String, MeshHandle>,
    /// The memory allocator this registry should use.
    allocator: Arc<dyn MemoryAllocator>,
    /// The vertex buffer allocated for all registered meshes.
    pub vertex_buffer: Option<VertexBuffer>,
    /// The index buffer allocated for all registered meshes.
    pub index_buffer: Option<IndexBuffer>,
    /// The raw vertex data backing `vertex_buffer`, kept around so a single mesh can be appended
    /// via [`Self::register_mesh`] without losing every previously registered mesh's data.
    raw_vertices: Vec<Vertex3>,
    /// The raw index data backing `index_buffer`. See `raw_vertices`.
    raw_indices: Vec<u32>,
}

impl MeshRegistry {
    /// Creates a new, blank `MeshRegistry` instance.
    pub fn new(allocator: Arc<dyn MemoryAllocator>) -> Self {
        Self {
            meshes: HashMap::new(),
            vertex_buffer: None,
            index_buffer: None,
            raw_vertices: Vec::new(),
            raw_indices: Vec::new(),
            allocator
        }
    }

    /// Appends a single already-loaded mesh to `verts`/`indices`, returning a [`MeshHandle`]
    /// describing where its data landed. A submesh's `material_id` is derived purely from
    /// `mesh_id` and the submesh's position in the source file, so this works whether or not a
    /// material has actually been registered under that id yet.
    fn append_mesh(
        mesh_id: &str,
        info: MeshLoadInfo,
        verts: &mut Vec<Vertex3>,
        indices: &mut Vec<u32>,
    ) -> MeshHandle {
        let vertex_loc = verts.len();
        let vertex_count = info.vertices.len();
        info.vertices
            .iter()
            .zip(info.normals.iter())
            .zip(info.uvs.iter())
            .for_each(|((pos, norm), uv)| {
                verts.push(Vertex3::new(*pos, *norm, *uv));
            });

        let index_loc = indices.len();
        let index_count = info.indices.len();
        indices.extend(info.indices);

        let submeshes = info
            .submeshes
            .into_iter()
            .map(|submesh| SubmeshHandle {
                material_id: submesh
                    .material_index
                    .map(|i| format!("{mesh_id}_mat_{i}")),
                index_loc: index_loc + submesh.index_start,
                index_count: submesh.index_count,
            })
            .collect::<Vec<_>>();

        println!(
            "Loaded mesh \"{}\" with {} vertices, {} indices, and {} submesh(es)",
            mesh_id, vertex_count, index_count, submeshes.len()
        );

        MeshHandle {
            id: mesh_id.to_string(),
            vertex_loc,
            vertex_count,
            index_loc,
            index_count,
            submeshes,
        }
    }

    /// Registers a single mesh, independently of a full [`Scene`] - e.g. from
    /// [`crate::assets::asset_manager::AssetManager`] loading one model at a time. Appends the
    /// mesh's data to the registry's shared buffers and reallocates them.
    ///
    /// This is far less efficient than batching a whole scene through [`Self::load_scene`], since
    /// every call reallocates and re-uploads the full vertex/index buffers - fine for loading a
    /// handful of models up front, not for anything performance-sensitive.
    pub fn register_mesh(&mut self, mesh_id: String, info: MeshLoadInfo) {
        let handle = Self::append_mesh(
            &mesh_id,
            info,
            &mut self.raw_vertices,
            &mut self.raw_indices,
        );
        self.meshes.insert(mesh_id, handle);
        self.rebuild_buffers();
    }

    /// (Re)allocates `vertex_buffer`/`index_buffer` from the currently accumulated
    /// `raw_vertices`/`raw_indices`.
    fn rebuild_buffers(&mut self) {
        self.vertex_buffer = Some(Self::alloc_vert_buffer(
            self.allocator.clone(),
            self.raw_vertices.clone(),
        ));
        self.index_buffer = Some(Self::alloc_index_buffer(
            self.allocator.clone(),
            self.raw_indices.clone(),
        ));
    }

    /// Allocates and writes to a vertex buffer with the given `Vec<Vertex3>`.
    fn alloc_vert_buffer(
        alloc: Arc<dyn MemoryAllocator>,
        verts: Vec<Vertex3>,
    ) -> VertexBuffer {
        println!("Allocating vertex buffer");
        Arc::new(
            Buffer::from_iter(
                alloc.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                verts,
            )
            .expect("Couldn't allocate vertex buffer"),
        )
    }

    /// Allocates and writes to an index buffer with the given `Vec<usize>`.
    fn alloc_index_buffer(
        alloc: Arc<dyn MemoryAllocator>,
        indices: Vec<u32>,
    ) -> IndexBuffer {
        println!("Allocating index buffer");
        Arc::new(
            Buffer::from_iter(
                alloc.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                indices,
            )
            .expect("Couldn't allocate index buffer"),
        )
    }

    /// Attempts to find a [`MeshHandle`] with the given ID, returning `None` if it's not found.
    pub fn get(&self, uuid: &String) -> Option<&MeshHandle> {
        self.meshes.get(uuid)
    }

    /// Returns a boolean indicating whether the requested [`MeshHandle`] exists.
    pub fn exists(&self, uuid: &String) -> bool {
        self.get(uuid).is_some()
    }
}
