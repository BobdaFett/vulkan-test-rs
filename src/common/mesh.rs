use crate::common::scene::Scene;
use crate::gpu::vertex3::Vertex3;
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};
use wavefront::Obj;

/// This struct is slightly misleading - it doesn't actually contain the mesh information, but
/// the locations of the mesh's information in the overall application's vertex buffer, index buffer,
/// and other such structures.
///
/// Once loaded, the mesh should be accessible via ID through a registry-type struct, which simply
/// maintains the list of meshes that have been loaded by the application so far and which assigns
/// the correct indices for buffer-to-render logic.
///
/// That means that this `Mesh` struct is actually just an entry within that registry. They should
/// rarely be used directly, and should never be edited.
#[derive(Debug)]
pub struct Mesh {
    pub id: String,
    pub vertex_loc: usize,
    pub vertex_count: usize,
    pub index_loc: usize,
    pub index_count: usize,
}

impl Mesh {}

/// # Mesh Registry
/// A wrapper around a series of [`Mesh`] structs, keyed by a unique ID, and the buffers that are
/// used by the rendering pipeline.
#[derive(Debug)]
pub struct MeshRegistry {
    pub meshes: HashMap<String, Mesh>,
    allocator: Arc<dyn MemoryAllocator>,
    pub vertex_buffer: Arc<Subbuffer<[Vertex3]>>,
    pub index_buffer: Arc<Subbuffer<[u32]>>,
}

impl MeshRegistry {
    /// Creates a `MeshRegistry` from the given [`Scene`].
    ///
    /// This method will load all the meshes indicated by the scene and allocate the required
    /// buffers. This significantly simplifies the allocation process, as the buffers only need to
    /// be allocated once.
    pub fn from_scene(scene: &Scene, allocator: Arc<dyn MemoryAllocator>) -> Self {
        // Load all the meshes into a single map.
        println!("Loading meshes from scene");
        let meshes = scene
            .mesh_paths
            .iter()
            .map(|(id, path)| {
                (
                    id.clone(),
                    Obj::from_file(path).expect("Couldn't read wavefront file"),
                )
            })
            .collect::<HashMap<String, Obj>>();

        let mut buf_verts = Vec::new();
        let mut buf_indices = Vec::new();
        let mut mesh_list = HashMap::new();
        meshes.into_iter().for_each(|(id, obj)| {
            // Information will be added directly to the verts and indices vectors, and a new Mesh
            // struct will be created and inserted into the mesh_list.
            let vert_start = buf_verts.len();
            let all_verts = obj.positions();
            let all_norms = obj.normals();
            let obj_num_verts = all_verts.len();
            all_verts.iter().zip(all_norms.iter())
                .zip(obj.uvs().iter())
                .for_each(|((pos, norm), uv)| {
                    buf_verts.push(Vertex3::new(*pos, *norm, *uv));
                });
            // buf_verts.extend(all_verts.map(|v| Vertex3::new(v.position(), v.normal().unwrap_or_default())));

            let index_list = obj
                .triangles()
                .flat_map(|t| {
                    t.iter()
                        .map(|i| i.position_index() as u32)
                        .collect::<Vec<u32>>()
                })
                .collect::<Vec<u32>>();
            let index_start = buf_indices.len();
            let obj_num_idx = index_list.len();
            buf_indices.extend(index_list);

            println!("Loaded mesh {} with {} vertices and {} indices", id, obj_num_verts, obj_num_idx);

            // TODO Create bounding boxes for each mesh and associate them with the struct.
            mesh_list.insert(
                id.clone(),
                Mesh {
                    id,
                    vertex_loc: vert_start,
                    vertex_count: obj_num_verts,
                    index_loc: index_start,
                    index_count: obj_num_idx,
                },
            );
        });

        // Allocate buffers for the information.
        let vertex_buffer = Self::alloc_vert_buffer(allocator.clone(), buf_verts);
        let index_buffer = Self::alloc_index_buffer(allocator.clone(), buf_indices);

        // Finally, return the struct.
        Self {
            meshes: mesh_list,
            vertex_buffer,
            index_buffer,
            allocator,
        }
    }

    /// Allocates and writes to a vertex buffer with the given `Vec<Vertex3>`.
    fn alloc_vert_buffer(
        alloc: Arc<dyn MemoryAllocator>,
        verts: Vec<Vertex3>,
    ) -> Arc<Subbuffer<[Vertex3]>> {
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
    ) -> Arc<Subbuffer<[u32]>> {
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

    /// Attempts to find a [`Mesh`] with the given ID, returning `None` if it's not found.
    pub fn get(&self, uuid: &String) -> Option<&Mesh> {
        self.meshes.get(uuid)
    }
}
