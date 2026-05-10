use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};
use crate::common::instance::InstanceRegistry;
use crate::common::mesh::MeshRegistry;
use crate::gpu::instance::GpuInstance;

#[derive(Debug, Clone)]
pub struct RenderBatch {
    pub mesh_id: String,
    pub instance_buffer: Subbuffer<[GpuInstance]>,
    pub instance_count: u32,
}

impl RenderBatch {
    /// Builds a `Vec<RenderBatch>` from the given list of instances and meshes. This should be done
    /// once per frame.
    pub fn build_batches(
        allocator: Arc<dyn MemoryAllocator>,
        mesh_registry: Arc<MeshRegistry>,
        instance_registry: Arc<InstanceRegistry>
    ) -> Vec<Self> {
        mesh_registry.meshes.iter()
            .map(|(id, mesh)| {
                let instances = instance_registry.get_instances_for(id);

                let gpu_instances = instances.iter()
                    .cloned()
                    .map(|instance| {
                        instance.into()
                    })
                    .collect::<Vec<GpuInstance>>();

                // Build instances into a RenderBatch
                let instance_buffer = Buffer::from_iter(
                    allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::VERTEX_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                            MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    gpu_instances
                ).expect("Couldn't allocate instance buffer");

                Self {
                    mesh_id: id.clone(),
                    instance_buffer,
                    instance_count: instances.len() as u32,
                }
            }).collect()
    }
}
