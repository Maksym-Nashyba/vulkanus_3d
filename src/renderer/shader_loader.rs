use std::sync::Arc;
use vulkano::device::Device;
use vulkano::shader::{ShaderCreationError, ShaderModule};

pub struct ShaderContainer{
    shaders: Vec<LoadedShader>
}

#[derive(PartialEq, Eq, Clone)]
pub enum ShaderType{
    Vertex,
    Fragment
}

#[derive(Clone)]
struct LoadedShader{
    shader:Arc<ShaderModule>,
    shader_type: ShaderType,
    name: String
}

mod direct_vert {
    vulkano_shaders::shader!{
        ty: "vertex",
        path : "src/shaders/direct.vert"
    }
}

mod direct_frag {
    vulkano_shaders::shader!{
        ty: "fragment",
        path : "src/shaders/direct.frag"
    }
}

impl ShaderContainer{
    pub fn load(device: Arc<Device>) -> Result<ShaderContainer, ShaderCreationError>{
        let mut loaded_shaders: Vec<LoadedShader> = Vec::new();

        loaded_shaders.push(LoadedShader{
            name:String::from("direct"),
            shader_type:ShaderType::Vertex,
            shader: direct_vert::load(device.clone())?
        });

        loaded_shaders.push(LoadedShader{
            name:String::from("direct"),
            shader_type:ShaderType::Fragment,
            shader: direct_frag::load(device.clone())?
        });

        return Ok(ShaderContainer{
            shaders:loaded_shaders});
    }

    pub fn get_shader(&self, shader_type:ShaderType, name:&str) -> Option<Arc<ShaderModule>>{
        for shader in &self.shaders {
            if shader.shader_type == shader_type && shader.name.eq(name) {
                return Some(shader.shader.clone());
            }
        }
        return None;
    }
}