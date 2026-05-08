use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use nalgebra::Vector3;
use uuid::Uuid;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};
use wavefront::Obj;
use crate::common::scene::Scene;
use crate::gpu::vertex3::Vertex3;

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
    pub index_loc: usize,
}

impl Mesh {
    pub fn new(
        id: impl Into<String>,
        vertex_loc: usize,
        index_loc: usize,
    ) -> Self {
        Self {
            id: id.into(),
            vertex_loc,
            index_loc,
        }
    }
}

/// # Mesh Registry
/// A wrapper around a series of [`Mesh`] structs, keyed by a unique ID, and the buffers that are
/// used by the rendering pipeline.
#[derive(Debug)]
pub struct MeshRegistry {
    meshes: HashMap<String, Mesh>,
    allocator: Arc<dyn MemoryAllocator>,
    pub vertex_buffer: Arc<Subbuffer<[Vertex3]>>,
    pub index_buffer: Arc<Subbuffer<[u32]>>,
}

impl MeshRegistry {
    pub fn new(
        allocator: Arc<dyn MemoryAllocator>,
    ) -> Result<Self, Box<dyn Error>> {
        let vertex_buffer = Arc::new(Buffer::new_slice::<Vertex3>(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST |
                    MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            1_000_000,
        )?);

        let index_buffer = Arc::new(Buffer::new_slice::<u32>(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST |
                    MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            3_000_000,
        )?);

        Ok(Self {
            meshes: HashMap::new(),
            allocator,
            vertex_buffer,
            index_buffer,
        })
    }

    /// Creates a `MeshRegistry` from the provided [`Scene`].
    ///
    /// This method will load all the meshes indicated by the scene and allocate the required
    /// buffers. This significantly simplifies the allocation process, as the buffers only need to
    /// be allocated once.
    pub fn from_scene(scene: &Scene, allocator: Arc<dyn MemoryAllocator>) -> Self {
        // First, load all the meshes into a single map.
        println!("Loading meshes from scene");
        let meshes = scene.mesh_paths.iter().map(|(id, path)|
             (id.clone(), Obj::from_file(path).expect("Couldn't read wavefront file"))
        ).collect::<HashMap<String, Obj>>();

        // Second, build two more maps that contain the vertex and index information.
        let verts = meshes.iter().map(|(id, obj)|
            (id.clone(), obj.vertices().map(|v| v.position().into()).collect())
        ).collect::<HashMap<String, Vec<Vector3<f32>>>>();

        let indices = meshes.iter().map(|(id, obj)|
            (id.clone(), obj.vertices().map(|v| v.position_index()).collect())
        ).collect::<HashMap<String, Vec<usize>>>();

        // Apply static transformations here. The general idea is that we don't want to have to read
        // any information from the buffer without the correct information. We do want to keep track
        // of what mesh is at what location in the list though.
        println!("Applying static mesh transformations");

        // Get Vector3's as a list of Vertex3's (CPU to GPU)

        // Third, allocate and fill buffers with this information.
        // let vert_buffer = Self::alloc_vert_buffer(allocator, verts.values().collect());

        // Fourth, return the struct.

        todo!()
    }

    fn alloc_vert_buffer(
        alloc: Arc<dyn MemoryAllocator>,
        verts: Vec<Vertex3>
    ) -> Arc<Subbuffer<[Vertex3]>> {
        println!("Allocating vertex buffer");
        Arc::new(Buffer::from_iter(
            alloc.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST |
                    MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            verts
        ).expect("Couldn't allocate vertex buffer"))
    }

    /// Attempts to find a [`Mesh`] with the given ID, returning `None` if it's not found.
    pub fn get(&self, uuid: &str) -> Option<&Mesh> {
        self.meshes.get(uuid)
    }
}
