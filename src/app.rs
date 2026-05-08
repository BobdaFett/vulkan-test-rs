use std::error::Error;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::descriptor_set::allocator::{DescriptorSetAllocator, StandardDescriptorSetAllocator};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::{swapchain, sync, Validated, VulkanError, VulkanLibrary};
use vulkano::shader::ShaderModule;
use vulkano::sync::GpuFuture;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use crate::common::mesh::MeshRegistry;
use crate::gpu::vertex3::Vertex3;
use crate::triangle;

struct VulkanContext {
    window: Arc<Window>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    swapchain_images: Vec<Arc<Image>>,
    mem_allocator: Arc<dyn MemoryAllocator>,
    cmd_allocator: Arc<StandardCommandBufferAllocator>,
    desc_allocator: Arc<dyn DescriptorSetAllocator>,
    vert_shader: Arc<ShaderModule>,
    frag_shader: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    cmd_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
    mesh_registry: Arc<MeshRegistry>,
}

impl VulkanContext {
    /// Initializes and returns a new `VulkanContext`. It may fail in all cases where a Vulkan object
    /// may fail, specifically during initialization of shaders, devices, buffers, or any other such
    /// structures.
    pub fn init(instance: Arc<Instance>, event_loop: &ActiveEventLoop) -> Result<Self, Box<dyn Error>> {
        let attributes = WindowAttributes::default()
            .with_title("Vulkano Example")
            .with_resizable(true);

        let window = Arc::new(event_loop.create_window(attributes)?);

        let surface = Surface::from_window(instance.clone(), window.clone())?;

        let dev_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        // Get a physical device. Fails if there is no suitable device found. The device must have a
        // graphics queue and swapchain capabilities.
        let (physical_device, queue_family_idx) = Self::get_physical_device(
            instance.clone(),
            surface.clone(),
            &dev_extensions
        )?;

        // Create a Vulkan device. This is a handle to the connection between the program and the
        // chosen physical device.
        let (device, mut queues) = Self::create_device(
            physical_device.clone(),
            &dev_extensions,
            queue_family_idx as usize
        )?;

        // Get the first queue. This is what we'll use for drawing.
        let queue = queues.next()
            .ok_or("Queues list was empty.")?;

        let capabilities = physical_device
            .surface_capabilities(
                &surface,
                Default::default()
            )?;

        let dimensions = window
            .inner_size();
        let composite_alpha = capabilities
            .supported_composite_alpha
            .into_iter()
            .next()
            .ok_or("No composite alpha supported")?;
        let image_format = physical_device
            .surface_formats(&surface, Default::default())?
            [0]
            .0;

        // Swapchain needs to be easily changeable, since resizing the window creates a whole new
        // set of information. Putting this into a function would be good, but also the new
        // information comes from the window event handler.
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: capabilities.min_image_count + 1,
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            }
        )?;

        let mem_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let cmd_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));
        let desc_allocator = Arc::new(StandardDescriptorSetAllocator::new(device.clone(), Default::default()));

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {},
            }
        )?;

        let vert_shader = triangle::load_vertex(device.clone())
            .map_err(|e| format!("Failed to load vertex shader: {e}"))?;
        let frag_shader = triangle::load_fragment(device.clone())
            .map_err(|e| format!("Failed to load fragment shader: {e}"))?;

        let framebuffers = Self::create_framebuffers(&images, &render_pass)?;

        let image_extents = swapchain.image_extent();
        let pipeline = Self::create_pipeline(
            &device,
            &vert_shader,
            &frag_shader,
            &render_pass,
            [image_extents[0] as f32, image_extents[1] as f32]
        )?;

        let vertex_buffer = Arc::new(Buffer::from_iter(
            mem_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vec![
                Vertex3 { position: [0.5, 0.5, 0.0] },
                Vertex3 { position: [-0.5, 0.5, 0.0] },
                Vertex3 { position: [-0.5, -0.5, 0.0] },
                Vertex3 { position: [0.5, -0.5, 0.0] },
            ]
        )?);

        let index_buffer = Arc::new(Buffer::from_iter(
            mem_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vec![
                0, 1, 2,
                0, 2, 3,
            ]
        )?);

        let cmd_buffers = Self::get_command_buffers(
            &cmd_allocator,
            &queue,
            &pipeline,
            &framebuffers,
            &vertex_buffer,
            &index_buffer
        )?;

        let mesh_registry = Arc::new(MeshRegistry::new(
            mem_allocator.clone(),
        )?);

        Ok(Self {
            window,
            device,
            queue,
            swapchain,
            swapchain_images: images,
            mem_allocator,
            cmd_allocator,
            desc_allocator,
            render_pass,
            vert_shader,
            frag_shader,
            framebuffers,
            pipeline,
            cmd_buffers,
            mesh_registry,
        })
    }

    /// Draws a frame on the current [`Surface`].
    pub fn draw_frame(&mut self) -> Result<(), Box<dyn Error>> {
        let (image_idx, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None)
                .map_err(Validated::unwrap){
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    println!("Swapchain is out of date!");
                    self.recreate_swapchain(true)?;
                    return Ok(());
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        // Swapchain images may be suboptimal, which means that the swapchain is not in a good state.
        // This is fixed by recreating the swapchain, but we'll have to skip this frame.
        if suboptimal {
            println!("Swapchain is suboptimal!");
            self.recreate_swapchain(false)?
        }

        let execution = sync::now(self.device.clone())
            .join(acquire_future)
            .then_execute(self.queue.clone(), self.cmd_buffers[image_idx as usize].clone())?
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_idx),
            )
            .then_signal_fence_and_flush();

        Ok(())
    }

    /// Gets the correct [`PhysicalDevice`], which needs to match these requirements:
    ///  - Must have the correct `khr_swapchain` extension
    ///  - Must contain a graphics queue
    ///  - Priority is given to a discrete GPU, then the CPU.
    fn get_physical_device(
        instance: Arc<Instance>,
        surface: Arc<Surface>,
        extensions: &DeviceExtensions
    ) -> Result<(Arc<PhysicalDevice>, u32), String> {
        instance
            .enumerate_physical_devices()
            .map_err(|e| format!("Failed to enumerate physical devices: {e:?}"))?
            .filter(|dev| dev.supported_extensions().contains(extensions))
            .filter_map(|dev| {
                dev.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && dev.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|q| (dev, q as u32))
            })
            .min_by_key(|(dev, _)| {
                match dev.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::Cpu => 1,
                    _ => 2
                }
            })
            .ok_or_else(|| "No physical devices found".to_string())
    }

    /// Creates the Vulkan [`Device`] given a specific [`PhysicalDevice`].
    fn create_device(
        physical_device: Arc<PhysicalDevice>,
        device_extensions: &DeviceExtensions,
        graphics_index: usize
    ) -> Result<(Arc<Device>, impl ExactSizeIterator<Item = Arc<Queue>>), String> {
        Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: graphics_index as u32,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions.clone(),
                ..Default::default()
            }
        )
            .map_err(|e| format!("Failed to create device: {e}"))
    }

    /// Creates a new [`Framebuffer`] for each swapchain image.
    fn create_framebuffers(
        images: &Vec<Arc<Image>>,
        render_pass: &Arc<RenderPass>
    ) -> Result<Vec<Arc<Framebuffer>>, Box<dyn Error>> {
        let framebuffers = images.iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    }
                )
                    .unwrap()
            })
            .collect::<Vec<Arc<Framebuffer>>>();

        Ok(framebuffers)
    }

    /// Creates a [`GraphicsPipeline`] that corresponds to the given [`RenderPass`], vertex shader,
    /// and fragment shader.
    pub fn create_pipeline(
        device: &Arc<Device>,
        vert_shader: &Arc<ShaderModule>,
        frag_shader: &Arc<ShaderModule>,
        render_pass: &Arc<RenderPass>,
        extents: [f32; 2],
    ) -> Result<Arc<GraphicsPipeline>, Box<dyn Error>> {
        // Get shader stages
        let vs = vert_shader.entry_point("main")
            .ok_or("Failed to find vertex shader entry point")?;
        let fs = frag_shader.entry_point("main")
            .ok_or("Failed to find fragment shader entry point")?;

        let vertex_input_state = Vertex3::per_vertex()
            .definition(&vs)?;

        let stages = vec![
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or("Failed to create pipeline subpass")?;

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())?
        )?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: extents,
            depth_range: 0.0..=1.0,
        };

        Ok(GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                flags: Default::default(),
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default()
                )),
                dynamic_state: Default::default(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            }
        )?)
    }

    /// Creates a [`PrimaryAutoCommandBuffer`] for each swapchain image.
    fn get_command_buffers(
        cmd_allocator: &Arc<StandardCommandBufferAllocator>,
        queue: &Arc<Queue>,
        pipeline: &Arc<GraphicsPipeline>,
        framebuffers: &Vec<Arc<Framebuffer>>,
        vertex_buffer: &Arc<Subbuffer<[Vertex3]>>,
        index_buffer: &Subbuffer<[u32]>
    ) -> Result<Vec<Arc<PrimaryAutoCommandBuffer>>, Box<dyn Error>> {
        let buffers = framebuffers.iter()
            .filter_map(|framebuffer| {
                let mut builder = AutoCommandBufferBuilder::primary(
                    cmd_allocator.clone(),
                    queue.queue_family_index(),
                    CommandBufferUsage::MultipleSubmit
                ).ok()?;

                unsafe {
                    builder
                        .begin_render_pass(
                            RenderPassBeginInfo {
                                clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                            },
                            SubpassBeginInfo {
                                contents: SubpassContents::Inline,
                                ..Default::default()
                            }
                        ).ok()?
                        .bind_pipeline_graphics(pipeline.clone()).ok()?
                        .bind_vertex_buffers(0, vertex_buffer.as_ref().clone()).ok()?
                        .bind_index_buffer(index_buffer.clone()).ok()?
                        // .draw(vertex_buffer.len() as u32, 1, 0, 0).ok()?
                        .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0).ok()?
                        .end_render_pass(SubpassEndInfo::default()).ok()?;
                }

                builder.build().ok()
            })
            .collect();

        Ok(buffers)
    }

    /// Handles recreation logic for the [`Swapchain`] by using the current information and any new
    /// bounds. Will also handle resizing logic, as this requires extra information to be updated,
    /// specifically the [`GraphicsPipeline`] and [`PrimaryAutoCommandBuffer`]. All information is
    /// automatically set within the `VulkanContext`.
    pub fn recreate_swapchain(&mut self, resized: bool) -> Result<(), Box<dyn Error>> {
        unsafe { self.device.wait_idle()?; };

        let (new_swapchain, new_images) = self.swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: self.window.inner_size().into(),
                ..self.swapchain.create_info()
            })?;

        self.swapchain = new_swapchain;
        self.swapchain_images = new_images;

        // Recreate the framebuffers, which depend on the swapchain.
        let new_framebuffers = Self::create_framebuffers(&self.swapchain_images, &self.render_pass)?;

        self.framebuffers = new_framebuffers;

        if resized {
            let image_extents = self.swapchain.image_extent();
            let new_pipeline = Self::create_pipeline(
                &self.device,
                &self.vert_shader,
                &self.frag_shader,
                &self.render_pass,
                [image_extents[0] as f32, image_extents[1] as f32],
            )?;

            let command_buffers = Self::get_command_buffers(
                &self.cmd_allocator,
                &self.queue,
                &new_pipeline,
                &self.framebuffers,
                &self.mesh_registry.vertex_buffer,
                &self.mesh_registry.index_buffer
            )?;

            self.pipeline = new_pipeline;
            self.cmd_buffers = command_buffers;
        }

        Ok(())
    }
}

pub struct MainApplication {
    instance: Arc<Instance>,
    context: Option<VulkanContext>,
}

impl MainApplication {
    pub fn new(instance_extensions: InstanceExtensions) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            instance: Self::get_instance(instance_extensions)?,
            context: None,
        })
    }

    pub fn draw_frame(&mut self) -> Result<(), Box<dyn Error>> {
        // Draw a frame on the surface. This requires a VulkanContext, which in turn will handle
        // the rendering logic. We'll pass on the required information to the context, if it exists,
        // or we'll try to initialize the context.
        self.context
            .as_mut()
            .expect("Context is not initialized")
            .draw_frame()
    }

    /// Gets the Vulkan instance.
    fn get_instance(
        required_extensions: InstanceExtensions,
    ) -> Result<Arc<Instance>, String> {
        let library = VulkanLibrary::new()
            .map_err(|e| format!("Failed to find Vulkan library: {e}"))?;

        Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            }
        )
            .map_err(|e| format!("Failed to create instance: {e}"))
    }
}

impl ApplicationHandler for MainApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.context.is_none()
            .then(|| {
                // Initialization logic
                let context = VulkanContext::init(
                    self.instance.clone(),
                    event_loop
                ).expect("Couldn't initialize VulkanContext");

                self.context = Some(context);
            });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Closing window.");
                event_loop.exit();
            }
            WindowEvent::Resized(dimensions) => {
                println!("Window was resized, recreating swapchain. New dimensions: {:?}", dimensions);
                if let Some(context) = self.context.as_mut() {
                    context.recreate_swapchain(true).expect("Failed to recreate swapchain");
                }
            }
            WindowEvent::RedrawRequested => {
                // This is where all redrawing logic goes.
                self.draw_frame()
                    .unwrap_or_else(|e| println!("Failed to draw frame, dropping: {:?}", e));

                // Queue the next frame for drawing.
                self.context.as_ref().unwrap().window.request_redraw();
            }
            _ => ()
        }
    }
}
