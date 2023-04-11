use std::sync::Arc;
use winit::event_loop::EventLoop;
use winit::window::Window;
use crate::renderer::Renderer;

pub mod renderer;
pub mod material;
mod transform;

pub fn innit_renderer(window:Arc<Window>) -> Renderer{
    return renderer::initialize_renderer(window);
}