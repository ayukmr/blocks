use bytemuck::{Pod, Zeroable};
use std::mem;

// instance
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Instance {
    // position
    pos: [f32; 4],

    // texture
    tex: f32,

    // rendered faces
    faces: u32,
}

impl Instance {
    // layout attributes
    const ATTRS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32, 5 => Uint32];

    // create instance
    pub fn new(pos: [f32; 3], tex: f32, faces: [bool; 6]) -> Self {
        // convert bools into binary
        let faces_bin =
            faces
                .iter()
                .enumerate()
                .fold(0, |acc, (face, show)| {
                    if *show {
                        acc | (1 << face)
                    } else {
                        acc & !(1 << face)
                    }
                });

        Self {
            tex,
            pos:   [pos[0], pos[1], pos[2], 0.0],
            faces: faces_bin,
        }
    }

    // memory layout
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,

            // step as instances
            step_mode:  wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}
