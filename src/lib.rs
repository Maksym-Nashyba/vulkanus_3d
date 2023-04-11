use std::sync::Arc;
use vulkano::device::Device;
use vulkano::memory::allocator::{AllocationType, GenericMemoryAllocatorCreateInfo, StandardMemoryAllocator};
use winit::window::Window;
use crate::renderer::Renderer;

pub mod renderer;
pub mod material;

pub fn innit_renderer(window:Arc<Window>) -> Renderer{
    return renderer::initialize_renderer(window);
}

pub fn new_allocator(device:Arc<Device>) -> StandardMemoryAllocator{
    return StandardMemoryAllocator::new_default(device);
}