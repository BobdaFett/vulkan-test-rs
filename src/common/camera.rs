use std::f32::consts::PI;
use std::sync::Arc;
use nalgebra::{Matrix4, Point3, Vector2, Vector3};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::descriptor_set::allocator::DescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter};
use vulkano::pipeline::graphics::viewport::Viewport;
use crate::gpu::camera::CameraUniform;

/// A camera set within 3D space.
pub struct Camera {
    /// The position of the camera.
    pub position: Vector3<f32>,
    /// The forward direction of the camera.
    forward: Vector3<f32>,
    /// The upward direction of the camera.
    up: Vector3<f32>,
    /// The rightward direction of the camera.
    right: Vector3<f32>,
    /// The field of view, in radians.
    fov: f32,
    /// The extents of the viewport.
    extents: Vector2<u32>,
}

impl Camera {
    /// Creates a new camera. This default camera is positioned at (1, 1, 1) and is looking at
    /// (0, 0, 0), and a default FOV of 70 degrees.
    pub fn new(
        extents: impl Into<Vector2<u32>>
    ) -> Self {
        let extents = extents.into();
        let position = Vector3::new(-10.0, 10.0, -10.0);
        let target = Vector3::zeros();

        // Traveling along this vector moves us toward the target.
        let forward = (target - position).normalize();

        // A world up vector.
        let up_world = Vector3::y_axis();

        // Traveling along this vector moves us to the right.
        let right = up_world.cross(&forward).normalize();

        // Traveling along this vector moves us upward.
        let up = forward.cross(&right).normalize();

        Self {
            position,
            forward,
            up,
            right,
            fov: 70.0 * PI / 180.0,
            extents,
        }
    }

    /// Moves the camera forward, or backward if given negative speed.
    pub fn move_forward(&mut self, speed: f32) {
        self.position += self.forward * speed;
    }

    /// Moves the camera to the right, or left if given negative speed.
    pub fn move_right(&mut self, speed: f32) {
        self.position += self.right * speed;
    }

    /// Moves the camera upwards, or downward if given negative speed.
    pub fn move_up(&mut self, speed: f32) {
        self.position += self.up * speed;
    }

    /// Sets the extents of the viewport.
    pub fn extents(&mut self, extents: impl Into<Vector2<u32>>) {
        let extents = extents.into();

        self.extents = extents;
    }

    /// Gets the current view matrix of the camera.
    pub fn get_view(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            &Point3::from(self.position),
            &Point3::from(self.forward - self.position),
            &self.up
        )
    }

    /// Creates a perspective matrix based on the current viewport aspect ratio.
    pub fn get_projection(&self) -> Matrix4<f32> {
        let aspect = self.extents.x as f32 / self.extents.y as f32;

        let mut proj = Matrix4::new_perspective(
            aspect,
            self.fov,
            1.0,
            10000.0
        );

        // Flip the y-axis, since Vulkan uses a different coordinate system.
        proj[(1, 1)] *= -1.0;

        proj
    }

    /// Creates a [`Viewport`] based on the current extents.
    pub fn viewport(&self) -> Viewport {
        let extent = self.extents.map(|v| v as f32).into();
        Viewport {
            offset: [0.0, 0.0],
            extent,
            depth_range: 0.0..=1.0,
        }
    }
}

/// An intermediate struct that maintains the buffer and descriptor set for the [`Camera`] struct.
/// The reason this information is not stored as part of the `Camera` itself is because tying
/// the GPU or rendering resources to the camera itself is quite awkward.
pub struct CameraResources {
    buffer: Arc<Subbuffer<CameraUniform>>,
    descriptor_set: Arc<DescriptorSet>,
}

impl CameraResources {
    /// Creates a new set of `CameraResources`. Automatically allocates a uniform buffer to store
    /// [`CameraUniform`] information.
    pub fn new(
        alloc: Arc<dyn MemoryAllocator>,
        desc_alloc: Arc<dyn DescriptorSetAllocator>,
        desc_layout: Arc<DescriptorSetLayout>,
    ) -> Arc<Self> {
        // Allocate the buffer
        let buffer = Arc::new(Buffer::new_sized::<CameraUniform>(
            alloc.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST |
                    MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            }
        ).expect("Couldn't allocate the camera uniform buffer"));

        // Create the descriptor set
        let descriptor_set = DescriptorSet::new(
            desc_alloc.clone(),
            desc_layout,
            [WriteDescriptorSet::buffer(0, buffer.as_ref().clone())],
            []
        ).expect("Couldn't create camera descriptor set");

        Arc::new(Self {
            buffer,
            descriptor_set
        })
    }

    /// Updates the buffer with the given [`Camera`] information.
    pub fn update(&self, camera: &Camera) {
        // Update the buffer with the new information from the camera.
        let view = camera.get_view();
        let proj = camera.get_projection();

        let mut buffer_data = self.buffer.write()
            .unwrap();
        buffer_data.proj = proj.into();
        buffer_data.view = view.into();
    }

    /// Get a reference to the descriptor set.
    pub fn descriptor_set(&self) -> Arc<DescriptorSet> {
        self.descriptor_set.clone()
    }
}
