use std::sync::Arc;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::shader::ShaderModule;
use crate::renderer::Renderer;

#[derive(Clone)]
pub struct Material{
    pipeline:Arc<GraphicsPipeline>
}

impl Material {
    pub fn new(renderer:&Renderer, vertex_shader:Arc<ShaderModule>, fragment_shader:Arc<ShaderModule>) -> Self{
        let pipeline:Arc<GraphicsPipeline>
            = renderer.build_pipeline(vertex_shader.clone(), fragment_shader.clone());
        return Self{
            pipeline:pipeline
        };
    }

    pub fn pipeline(&self) -> Arc<GraphicsPipeline>{
        return self.pipeline.clone();
    }
}