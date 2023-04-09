use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::impl_vertex;
use vulkano::memory::allocator::StandardMemoryAllocator;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex{
    pub position: [f32; 3]
}

impl_vertex!(Vertex, position);

#[derive(Clone)]
pub struct Model{
    pub buffer: Arc<CpuAccessibleBuffer<[Vertex]>>
}

impl Model {
    pub fn load(memory_allocator: &StandardMemoryAllocator, vertices:Vec<Vertex>) -> Model{
        let vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>> = CpuAccessibleBuffer::from_iter(
            memory_allocator,
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            false,
            vertices,
        ).unwrap();

        return Model{
            buffer:vertex_buffer
        }
    }
}