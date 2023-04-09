use winit::event_loop::EventLoop;
use crate::renderer::Renderer;

pub mod renderer;
mod transform;
mod material;

pub fn innit_renderer(event_loop:&EventLoop<()>) -> Renderer{
    return renderer::initialize_renderer(&event_loop);
}