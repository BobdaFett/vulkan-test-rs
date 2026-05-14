use crate::common::mesh::MeshRegistry;
use crate::gpu::vertex3::Vertex3;
use crate::triangle;
use std::error::Error;
use std::sync::Arc;
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents, SubpassEndInfo,
};
use vulkano::descriptor_set::allocator::{DescriptorSetAllocator, StandardDescriptorSetAllocator};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, StandardMemoryAllocator};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::{Validated, VulkanError, VulkanLibrary, swapchain, sync};
use vulkano::format::Format;
use vulkano::pipeline::graphics::depth_stencil::{DepthState, DepthStencilState};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use crate::common::camera::{Camera, CameraResources};
use crate::common::instance::InstanceRegistry;
use crate::common::render_batch::RenderBatch;
use crate::common::scene::Scene;
use crate::gpu::instance::GpuInstance;

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
    mesh_registry: Arc<MeshRegistry>,
    instance_registry: Arc<InstanceRegistry>,
    camera: Camera,
    camera_resources: Arc<CameraResources>,
}

impl VulkanContext {
    /// Initializes and returns a new `VulkanContext`. It may fail in all cases where a Vulkan object
    /// may fail, specifically during initialization of shaders, devices, buffers, or any other such
    /// structures.
    pub fn init(
        instance: Arc<Instance>,
        event_loop: &ActiveEventLoop,
    ) -> Result<Self, Box<dyn Error>> {
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
        let (physical_device, queue_family_idx) =
            Self::get_physical_device(instance.clone(), surface.clone(), &dev_extensions)?;

        // Create a Vulkan device. This is a handle to the connection between the program and the
        // chosen physical device.
        let (device, mut queues) = Self::create_device(
            physical_device.clone(),
            &dev_extensions,
            queue_family_idx as usize,
        )?;

        // Get the first queue. This is what we'll use for drawing.
        let queue = queues.next().ok_or("Queues list was empty.")?;

        let capabilities = physical_device.surface_capabilities(&surface, Default::default())?;

        let dimensions = window.inner_size();
        let composite_alpha = capabilities
            .supported_composite_alpha
            .into_iter()
            .next()
            .ok_or("No composite alpha supported")?;
        let image_format = physical_device.surface_formats(&surface, Default::default())?[0].0;

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
            },
        )?;

        let mem_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let cmd_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));
        let desc_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
                depth: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth},
            }
        )?;

        let vert_shader = triangle::load_vertex(device.clone())
            .map_err(|e| format!("Failed to load vertex shader: {e}"))?;
        let frag_shader = triangle::load_fragment(device.clone())
            .map_err(|e| format!("Failed to load fragment shader: {e}"))?;

        let framebuffers = Self::create_framebuffers(
            mem_allocator.clone(),
            &images,
            &render_pass
        )?;

        let image_extents = swapchain.image_extent();

        // Create the camera.
        let camera = Camera::new(image_extents);

        let pipeline = Self::create_pipeline(
            &device,
            &vert_shader,
            &frag_shader,
            &render_pass,
        )?;

        // Get descriptor set layout
        let pipeline_layout = pipeline.layout().clone();
        let desc_set_layout = pipeline_layout.set_layouts()[0].clone();

        // Create GPU-based camera resources
        let camera_resources = CameraResources::new(
            mem_allocator.clone(),
            desc_allocator.clone(),
            desc_set_layout
        );

        let scene = Scene::from_file_json("./test_scene.scene");
        let mesh_registry = Arc::new(MeshRegistry::from_scene(
            &scene,
            mem_allocator.clone()
        ));
        let instance_registry = Arc::new(InstanceRegistry::from_scene(
            &scene
        ));

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
            mesh_registry,
            instance_registry,
            camera,
            camera_resources
        })
    }

    /// Draws a frame on the current [`Surface`].
    pub fn draw_frame(&mut self) -> Result<(), Box<dyn Error>> {
        // self.camera.move_right(0.1);
        self.camera_resources.update(&self.camera);
        let (image_idx, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
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

        // Rebuild command buffers
        let cmd_buffer = Self::get_command_buffer(
            &self.cmd_allocator,
            &self.queue,
            &self.pipeline,
            self.framebuffers[image_idx as usize].clone(),
            &self.camera,
            &self.camera_resources,
            self.mesh_registry.clone(),
            RenderBatch::build_batches(
                self.mem_allocator.clone(),
                self.mesh_registry.clone(),
                self.instance_registry.clone()
            )
        )?;

        let future = sync::now(self.device.clone())
            .join(acquire_future)
            .then_execute(
                self.queue.clone(),
                cmd_buffer.clone(),
            )?
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_idx),
            )
            .then_signal_fence_and_flush()
            .map_err(|e| format!("Failed to draw frame: {e}"))?;

        future.wait(None)?;

        Ok(())
    }

    /// Gets the correct [`PhysicalDevice`], which needs to match these requirements:
    ///  - Must have the correct `khr_swapchain` extension
    ///  - Must contain a graphics queue
    ///  - Priority is given to a discrete GPU, then the CPU.
    fn get_physical_device(
        instance: Arc<Instance>,
        surface: Arc<Surface>,
        extensions: &DeviceExtensions,
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
            .min_by_key(|(dev, _)| match dev.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::Cpu => 1,
                _ => 2,
            })
            .ok_or_else(|| "No physical devices found".to_string())
    }

    /// Creates the Vulkan [`Device`] given a specific [`PhysicalDevice`].
    fn create_device(
        physical_device: Arc<PhysicalDevice>,
        device_extensions: &DeviceExtensions,
        graphics_index: usize,
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
            },
        )
        .map_err(|e| format!("Failed to create device: {e}"))
    }

    /// Creates a new [`Framebuffer`] for each swapchain image.
    fn create_framebuffers(
        alloc: Arc<dyn MemoryAllocator>,
        images: &Vec<Arc<Image>>,
        render_pass: &Arc<RenderPass>,
    ) -> Result<Vec<Arc<Framebuffer>>, Box<dyn Error>> {
        let depth_buffer = ImageView::new_default(
            Image::new(
                alloc.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D16_UNORM,
                    extent: images[0].extent(),
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default()
            )?
        )?;

        let framebuffers = images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view, depth_buffer.clone()],
                        ..Default::default()
                    },
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
    ) -> Result<Arc<GraphicsPipeline>, Box<dyn Error>> {
        // Get shader stages
        let vs = vert_shader
            .entry_point("main")
            .ok_or("Failed to find vertex shader entry point")?;
        let fs = frag_shader
            .entry_point("main")
            .ok_or("Failed to find fragment shader entry point")?;

        let vertex_input_state = [Vertex3::per_vertex(), GpuInstance::per_instance()]
            .definition(&vs)?;

        let stages = vec![
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let subpass =
            Subpass::from(render_pass.clone(), 0).ok_or("Failed to create pipeline subpass")?;

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())?,
        )?;

        // This is a default viewport. It will be updated later via the camera struct in the context.
        let viewport = Viewport::default();

        Ok(GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
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
                    ColorBlendAttachmentState::default(),
                )),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )?)
    }

    /// Creates a [`PrimaryAutoCommandBuffer`] for the given framebuffer, render pass, and queue.
    /// This is used to pass instructions to draw a frame.
    fn get_command_buffer(
        cmd_allocator: &Arc<StandardCommandBufferAllocator>,
        queue: &Arc<Queue>,
        pipeline: &Arc<GraphicsPipeline>,
        framebuffer: Arc<Framebuffer>,
        camera: &Camera,
        camera_resources: &CameraResources,
        mesh_registry: Arc<MeshRegistry>,
        render_batches: Vec<RenderBatch>,
    ) -> Result<Arc<PrimaryAutoCommandBuffer>, Box<dyn Error>> {
        let mut builder = AutoCommandBufferBuilder::primary(
            cmd_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
            .expect("Couldn't create command buffer builder");

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.1, 0.1, 0.1, 0.5].into()),
                        Some(1f32.into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )?
            .set_viewport(
                0,
                [camera.viewport()].into_iter().collect()
            )?
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                camera_resources.descriptor_set().clone(),
            )?
            .bind_pipeline_graphics(pipeline.clone())?
            .bind_vertex_buffers(0, mesh_registry.vertex_buffer.as_ref().clone())?
            .bind_index_buffer(mesh_registry.index_buffer.as_ref().clone())?;

        unsafe {
            render_batches.iter().for_each(|batch| {
                // Grab mesh information
                let mesh = mesh_registry.get(&batch.mesh_id)
                    .expect("Mesh does not exist in registry");

                builder
                    .bind_vertex_buffers(1, batch.instance_buffer.clone())
                    .unwrap()
                    .draw_indexed(mesh.index_count as u32, batch.instance_count, mesh.index_loc as u32, mesh.vertex_loc as i32, 0)
                    .unwrap();
            });
        }

        builder.end_render_pass(SubpassEndInfo::default())?;

        let buffer = builder.build()?;

        Ok(buffer)
    }

    /// Handles recreation logic for the [`Swapchain`] by using the current information and any new
    /// bounds. Will also handle resizing logic, as this requires extra information to be updated,
    /// specifically the [`GraphicsPipeline`] and [`PrimaryAutoCommandBuffer`]. All information is
    /// automatically set within the `VulkanContext`.
    // TODO Define a function that handles creation of all information related to the size of the window.
    pub fn recreate_swapchain(&mut self, resized: bool) -> Result<(), Box<dyn Error>> {
        unsafe {
            self.device.wait_idle()?;
        };

        let image_extents = self.window.inner_size();

        let (new_swapchain, new_images) = self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: image_extents.clone().into(),
            ..self.swapchain.create_info()
        })?;

        self.swapchain = new_swapchain;
        self.swapchain_images = new_images;

        // Recreate the framebuffers, which depend on the swapchain.
        let new_framebuffers =
            Self::create_framebuffers(
                self.mem_allocator.clone(),
                &self.swapchain_images,
                &self.render_pass
            )?;

        self.framebuffers = new_framebuffers;

        if resized {
            let new_pipeline = Self::create_pipeline(
                &self.device,
                &self.vert_shader,
                &self.frag_shader,
                &self.render_pass,
            )?;

            self.camera.extents([image_extents.width, image_extents.height]);

            self.pipeline = new_pipeline;
        }

        Ok(())
    }
}

pub struct App {
    instance: Arc<Instance>,
    context: Option<VulkanContext>,
}

impl App {
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
    fn get_instance(required_extensions: InstanceExtensions) -> Result<Arc<Instance>, String> {
        let library =
            VulkanLibrary::new().map_err(|e| format!("Failed to find Vulkan library: {e}"))?;

        Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_string()],
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .map_err(|e| format!("Failed to create instance: {e}"))
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.context.is_none().then(|| {
            // Initialization logic
            let context = VulkanContext::init(self.instance.clone(), event_loop)
                .expect("Couldn't initialize VulkanContext");

            self.context = Some(context);
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Closing window.");
                event_loop.exit();
            }
            WindowEvent::Resized(dimensions) => {
                println!(
                    "Window was resized, recreating swapchain. New dimensions: {:?}",
                    dimensions
                );
                if let Some(context) = self.context.as_mut() {
                    context
                        .recreate_swapchain(true)
                        .expect("Failed to recreate swapchain");
                }
            }
            WindowEvent::RedrawRequested => {
                // This is where all redrawing logic goes.
                self.draw_frame()
                    .unwrap_or_else(|e| println!("Failed to draw frame, dropping: {:?}", e));

                // Queue the next frame for drawing.
                self.context.as_ref().unwrap().window.request_redraw();
            }
            _ => (),
        }
    }
}
