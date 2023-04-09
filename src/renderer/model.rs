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

    pub fn star(memory_allocator: &StandardMemoryAllocator) -> Model{
        let vertices = vec![
            Vertex {
                position: [-0.75, 1.0, 0.5],
            },
            Vertex {
                position: [0.0, -1.0, 0.5],
            },
            Vertex {
                position: [0.4, 0.0, 0.5],
            },
            Vertex {
                position: [-1.0, -0.5, 0.5],
            },
            Vertex {
                position: [0.2, -0.5, 0.5],
            },
            Vertex {
                position: [0.75, 1.0, 0.5],
            },
            Vertex {
                position: [0.2, -0.5, 0.5],
            },
            Vertex {
                position: [1.0, -0.5, 0.5],
            },
            Vertex {
                position: [0.4, 0.0, 0.5],
            },
        ];
        return Self::load(memory_allocator, vertices);
    }
}
