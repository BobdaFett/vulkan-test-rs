use crate::assets::loaders::mesh_loader::{MeshInfo, MeshLoader};
use crate::common::scene::Scene;
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
    /// The material index this submesh was assigned in its source file, if any. This is not yet
    /// resolved to a [`crate::common::material::Material`] - that mapping is the caller's
    /// responsibility until material loading is wired into mesh registration.
    pub material_index: Option<usize>,
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
}

impl MeshRegistry {
    /// Creates a new, blank `MeshRegistry` instance.
    pub fn new(allocator: Arc<dyn MemoryAllocator>) -> Self {
        Self {
            meshes: HashMap::new(),
            vertex_buffer: None,
            index_buffer: None,
            allocator
        }
    }

    /// Registers a [`MeshHandle`] to the registry, if it does not already exist.
    pub fn register(&mut self, mesh_id: String, mesh: MeshHandle) {
        self.meshes.insert(mesh_id, mesh);
    }

    /// Creates a `MeshRegistry` from the given [`Scene`].
    ///
    /// This method will load all the meshes indicated by the scene and allocate the required
    /// buffers. This significantly simplifies the allocation process, as the buffers only need to
    /// be allocated once.
    pub fn load_scene(&mut self, scene: &Scene, allocator: Arc<dyn MemoryAllocator>) {
        // Load all the meshes into a single map.
        println!("Loading meshes from scene");

        let mesh_loader = MeshLoader::new();
        let meshes = scene
            .mesh_paths
            .iter()
            .map(|(id, path)| {
                (
                    id.clone(),
                    mesh_loader
                        .load_mesh(path)
                        .expect("Couldn't read wavefront file"),
                )
            })
            .collect::<HashMap<String, MeshInfo>>();

        let mut buf_verts = Vec::new();
        let mut buf_indices = Vec::new();
        let mut mesh_list = HashMap::new();
        meshes.into_iter().for_each(|(id, obj)| {
            // Information will be added directly to the verts and indices vectors, and a new Mesh
            // struct will be created and inserted into the mesh_list.
            let vert_start = buf_verts.len();
            let all_verts = obj.vertices;
            let all_norms = obj.normals;
            let obj_num_verts = all_verts.len();
            all_verts
                .iter()
                .zip(all_norms.iter())
                .zip(obj.uvs.iter())
                .for_each(|((pos, norm), uv)| {
                    buf_verts.push(Vertex3::new(*pos, *norm, *uv));
                });

            let index_start = buf_indices.len();
            let obj_num_idx = obj.indices.len();
            buf_indices.extend(obj.indices);

            let submeshes = obj
                .submeshes
                .into_iter()
                .map(|submesh| SubmeshHandle {
                    material_index: submesh.material_index,
                    index_loc: index_start + submesh.index_start,
                    index_count: submesh.index_count,
                })
                .collect::<Vec<_>>();

            println!(
                "Loaded mesh \"{}\" with {} vertices, {} indices, and {} submesh(es)",
                id, obj_num_verts, obj_num_idx, submeshes.len()
            );

            // TODO Create bounding boxes for each mesh and associate them with the struct.
            mesh_list.insert(
                id.clone(),
                MeshHandle {
                    id,
                    vertex_loc: vert_start,
                    vertex_count: obj_num_verts,
                    index_loc: index_start,
                    index_count: obj_num_idx,
                    submeshes,
                },
            );
        });

        // Allocate buffers for the information.
        let vertex_buffer = Self::alloc_vert_buffer(allocator.clone(), buf_verts);
        let index_buffer = Self::alloc_index_buffer(allocator.clone(), buf_indices);

        self.meshes = mesh_list;
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
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
