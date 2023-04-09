use nalgebra_glm::{Quat, Vec3};

#[derive(Clone)]
pub struct Transform{
    pub position:Vec3,
    pub rotation:Quat,
    pub scale:Vec3
}

impl Transform {
    pub fn identity() -> Transform{
        return Transform{
            position:Vec3::identity(),
            rotation:Quat::identity(),
            scale:Vec3::identity()
        }
    }

    pub fn at_position(position: Vec3) -> Transform{
        return Transform{
            position: position,
            rotation:Quat::identity(),
            scale:Vec3::identity()
        }
    }
}