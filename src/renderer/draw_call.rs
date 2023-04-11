use nalgebra_glm::Mat4x4;
use crate::material::Material;
use crate::renderer::model::Model;

pub struct DrawCall{
    pub transform:Mat4x4,
    pub model:Model,
    pub material:Material
}