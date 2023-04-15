pub(crate) mod shader_loader;
pub mod draw_call;
pub mod model;

use std::sync::Arc;
use nalgebra_glm::Mat4x4;

use vulkano::{
    buffer::TypedBufferAccess,
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, physical::PhysicalDeviceType, QueueCreateInfo,
    },
    image::{ImageAccess, ImageUsage, SwapchainImage, view::ImageView},
    instance::{Instance, InstanceCreateInfo},
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        acquire_next_image, AcquireError, Swapchain, SwapchainCreateInfo, SwapchainCreationError,
        SwapchainPresentInfo,
    },
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary,
};
use vulkano::buffer::{BufferUsage, CpuBufferPool};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Queue;
use vulkano::memory::allocator::{MemoryUsage, StandardMemoryAllocator};
use vulkano::pipeline::{Pipeline, PipelineBindPoint};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{PresentMode, Surface, SwapchainAcquireFuture};
use vulkano::buffer::BufferContents;
use vulkano_win::create_surface_from_winit;
use winit::window::Window;
use crate::renderer::draw_call::DrawCall;
use crate::renderer::model::Vertex;
use crate::renderer::shader_loader::ShaderContainer;

pub struct Renderer{
    pub device: Arc<Device>,
    pub shader_container: ShaderContainer,
    pub(crate) allocator:StandardMemoryAllocator,
    render_surface: Arc<Surface>,
    swapchain_container: SwapchainContainer,
    render_pass: Arc<RenderPass>,
    queue: Arc<Queue>,
    viewport: Viewport,
    framebuffers: Vec<Arc<Framebuffer>>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
    uniform_buffer: CpuBufferPool<UniformData>,
    previous_frame_end: Option<Box<dyn GpuFuture>>
}

struct SwapchainContainer{
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<SwapchainImage>>,
    pub optimal: bool
}

#[derive(PartialEq, Eq, Clone)]
pub enum ShaderType{
    Vertex,
    Fragment
}

#[derive(BufferContents)]
#[repr(C)]
struct UniformData{
    transformation: Mat4x4
}

impl Renderer{
    pub fn new(window: Arc<Window>, present_immediate:bool) -> Self {
        let library = VulkanLibrary::new().unwrap();
        let required_extensions = vulkano_win::required_extensions(&library);

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                enumerate_portability: true,
                ..Default::default()
            },
        ).unwrap();

        let surface = create_surface_from_winit(window, instance.clone()).unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices().unwrap()
            .filter(|p| {
                p.supported_extensions().contains(&device_extensions)
            })
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.graphics && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| {
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                    _ => 5,
                }
            }).expect("No suitable physical device found");

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ).unwrap();

        let queue: Arc<Queue> = queues.next().unwrap();

        let (swapchain, images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();

            let image_format = Some(
                device
                    .physical_device()
                    .surface_formats(&surface, Default::default())
                    .unwrap()[0].0,
            );

            let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count,
                    image_format: image_format,
                    present_mode: if present_immediate{ PresentMode::Immediate }else{ PresentMode::Fifo },
                    image_extent: window.inner_size().into(),
                    image_usage: ImageUsage {
                        color_attachment: true,
                        ..ImageUsage::empty()
                    },
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                },
            ).unwrap()
        };

        let shader_container: ShaderContainer = ShaderContainer::load(device.clone()).unwrap();

        let render_pass: Arc<RenderPass> = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }).unwrap();

        let swapchain_container: SwapchainContainer =
            SwapchainContainer{
                swapchain: swapchain.clone(),
                images: images.clone(),
                optimal: true
            };

        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let framebuffers: Vec<Arc<Framebuffer>> = window_size_dependent_setup(&swapchain_container.images, render_pass.clone(), &mut viewport);

        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());

        let descriptor_set_allocator =
            StandardDescriptorSetAllocator::new(device.clone());

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        let uniform_buffer: CpuBufferPool<UniformData> = CpuBufferPool::<UniformData>::new(
            Arc::new(StandardMemoryAllocator::new_default(device.clone())),
            BufferUsage {
                uniform_buffer: true,
                ..BufferUsage::empty()
            },
            MemoryUsage::Upload,
        );

        return Self{
            device: device.clone(),
            shader_container: shader_container,
            render_surface: surface.clone(),
            swapchain_container: swapchain_container,
            render_pass: render_pass.clone(),
            queue: queue.clone(),
            viewport: viewport,
            framebuffers: framebuffers,
            allocator:StandardMemoryAllocator::new_default(device.clone()),
            command_buffer_allocator: command_buffer_allocator,
            descriptor_set_allocator: descriptor_set_allocator,
            uniform_buffer: uniform_buffer,
            previous_frame_end: previous_frame_end
        }
    }

    pub fn on_resized(&mut self) {
        self.swapchain_container.optimal = false;
    }

    pub fn submit_frame(&mut self, draw_calls:Vec<DrawCall>, block_until_drawn:bool){
        let window = self.render_surface.object().unwrap().downcast_ref::<Window>().unwrap();
        let dimensions = window.inner_size();
        if dimensions.width == 0 || dimensions.height == 0 {
            return;
        }

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if !self.swapchain_container.optimal {
            let (new_swapchain, new_images) =
                match self.swapchain_container.swapchain.recreate(SwapchainCreateInfo {
                    image_extent: dimensions.into(),
                    ..self.swapchain_container.swapchain.create_info()
                }) {
                    Ok(r) => r,
                    Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                    Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                };

            self.swapchain_container = SwapchainContainer{
                swapchain:new_swapchain.clone(),
                images: new_images.clone(),
                optimal: true
            };

            self.framebuffers = window_size_dependent_setup(
                &new_images,
                self.render_pass.clone(),
                &mut self.viewport,
            );
        }

        let (image_index, suboptimal, image_acquire_future) =
            match acquire_next_image(self.swapchain_container.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.swapchain_container.optimal = false;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };
        if suboptimal {
            self.swapchain_container.optimal = false;
        }

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();


        let uniform_data = UniformData{
            transformation: Mat4x4::identity()
        };
        let uniform_buffer_subbuffer=  self.uniform_buffer.from_data(uniform_data).unwrap();

        let uniform_descriptors = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)],
        ).unwrap();

        command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([1.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassContents::Inline,
            ).unwrap()
            .set_viewport(0, [self.viewport.clone()])
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                draw_calls.first().unwrap().pipeline().layout().clone(),
                0,
                uniform_descriptors);

            for draw_call in draw_calls {
                command_buffer_builder
                    .bind_pipeline_graphics(draw_call.material.pipeline())
                    .bind_vertex_buffers(0, draw_call.model.buffer.clone())
                    .draw(draw_call.model.buffer.len() as u32, 1, 0, 0).unwrap();
            }

        command_buffer_builder
            .end_render_pass().unwrap();

        let command_buffer = command_buffer_builder.build().unwrap();
        self.submit_command_buffer(command_buffer, image_acquire_future, image_index, block_until_drawn);
    }

    pub fn build_pipeline(&self, vertex_shader:Arc<ShaderModule>, fragment_shader:Arc<ShaderModule>) -> Arc<GraphicsPipeline>{
        return GraphicsPipeline::start()
            .render_pass(Subpass::from(self.render_pass.clone(), 0).unwrap())
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .input_assembly_state(InputAssemblyState::new())
            .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
            .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .build(self.device.clone()).unwrap();
    }

    fn submit_command_buffer(&mut self, command_buffer:PrimaryAutoCommandBuffer, image_acquire_future:SwapchainAcquireFuture, image_index:u32, block_until_drawn:bool){
        let future = self.previous_frame_end
            .take().unwrap()
            .join(image_acquire_future)
            .then_execute(self.queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain_container.swapchain.clone(), image_index), )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                if block_until_drawn {
                    future.wait(None).unwrap();
                }
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.swapchain_container.optimal = false;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                panic!("Failed to flush future: {:?}", e);
            }
        }
    }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
                .unwrap()
        })
        .collect::<Vec<_>>()
}