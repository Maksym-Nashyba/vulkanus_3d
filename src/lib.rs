use std::sync::Arc;
use winit::window::Window;
use crate::renderer::Renderer;

pub mod renderer;
pub mod material;

pub fn innit_renderer(window:Arc<Window>, present_immediate:bool) -> Renderer{
    return Renderer::new(window, present_immediate);
}