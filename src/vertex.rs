use std::mem;
use bytemuck::{Pod, Zeroable};

// texture sizing
pub const TEX_W: f32 = 1.0 / 6.0;
pub const TEX_H: f32 = 1.0 / 4.0;

// indices
pub const INDICES: &[u16] = &[
    // front
    0, 1, 2,
    3, 2, 1,

    // back
    6, 5, 4,
    5, 6, 7,

    // left
    8, 9, 10,
    11, 10, 9,

    // right
    12, 13, 14,
    15, 14, 13,

    // top
    16, 17, 18,
    19, 18, 17,

    // bottom
    22, 21, 20,
    21, 22, 23,
];

// get vertices
pub fn get_vertices() -> [Vertex; 24] {
    [
        // front face
        Vertex::new([-0.5, -0.5, 0.5], [0.0, 1.0], 0),
        Vertex::new([ 0.5, -0.5, 0.5], [1.0, 1.0], 0),
        Vertex::new([-0.5,  0.5, 0.5], [0.0, 0.0], 0),
        Vertex::new([ 0.5,  0.5, 0.5], [1.0, 0.0], 0),

        // back face
        Vertex::new([-0.5, -0.5, -0.5], [2.0, 1.0], 1),
        Vertex::new([ 0.5, -0.5, -0.5], [1.0, 1.0], 1),
        Vertex::new([-0.5,  0.5, -0.5], [2.0, 0.0], 1),
        Vertex::new([ 0.5,  0.5, -0.5], [1.0, 0.0], 1),

        // left face
        Vertex::new([-0.5, -0.5, -0.5], [2.0, 1.0], 2),
        Vertex::new([-0.5, -0.5,  0.5], [3.0, 1.0], 2),
        Vertex::new([-0.5,  0.5, -0.5], [2.0, 0.0], 2),
        Vertex::new([-0.5,  0.5,  0.5], [3.0, 0.0], 2),

        // right face
        Vertex::new([0.5, -0.5,  0.5], [3.0, 1.0], 3),
        Vertex::new([0.5, -0.5, -0.5], [4.0, 1.0], 3),
        Vertex::new([0.5,  0.5,  0.5], [3.0, 0.0], 3),
        Vertex::new([0.5,  0.5, -0.5], [4.0, 0.0], 3),

        // top face
        Vertex::new([-0.5, 0.5,  0.5], [4.0, 1.0], 4),
        Vertex::new([ 0.5, 0.5,  0.5], [5.0, 1.0], 4),
        Vertex::new([-0.5, 0.5, -0.5], [4.0, 0.0], 4),
        Vertex::new([ 0.5, 0.5, -0.5], [5.0, 0.0], 4),

        // bottom face
        Vertex::new([-0.5, -0.5,  0.5], [6.0, 1.0], 5),
        Vertex::new([ 0.5, -0.5,  0.5], [5.0, 1.0], 5),
        Vertex::new([-0.5, -0.5, -0.5], [6.0, 0.0], 5),
        Vertex::new([ 0.5, -0.5, -0.5], [5.0, 0.0], 5),
    ]
}

// single vertex
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    // position
    pos: [f32; 4],

    // texture position
    tex_pos: [f32; 2],

    // face index
    face: u32,
}

impl Vertex {
    // layout attributes
    const ATTRS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2, 2 => Uint32];

    // create vertex
    pub fn new(pos: [f32; 3], tex_pos: [f32; 2], face: u32) -> Self {
        Self {
            face,
            pos:     [pos[0], pos[1], pos[2], 1.0],
            tex_pos: [tex_pos[0] * TEX_W, tex_pos[1] * TEX_H],
        }
    }

    // memory layout
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode:    wgpu::VertexStepMode::Vertex,
            attributes:   &Self::ATTRS,
        }
    }
}
