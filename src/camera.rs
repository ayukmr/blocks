use crate::projection::Projection;
use bytemuck::{Pod, Zeroable};

use cgmath::{Point3, Vector3, Matrix4, Rad, SquareMatrix, InnerSpace};

// player camera
pub struct Camera {
    // position
    pub pos: Point3<f32>,

    // rotation
    pub yaw:   Rad<f32>,
    pub pitch: Rad<f32>,
}

impl Camera {
    // create camera
    pub fn new(pos: Point3<f32>, yaw: Rad<f32>, pitch: Rad<f32>) -> Self {
        Self { pos, yaw, pitch }
    }

    // calculate matrix
    pub fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw)     = self.yaw.0.sin_cos();

        // construct matrix
        Matrix4::look_to_rh(
            self.pos,

            Vector3::new(
                cos_pitch * cos_yaw,
                sin_pitch,
                cos_pitch * sin_yaw
            ).normalize(),

            Vector3::unit_y(),
        )
    }
}

// camera data in shaders
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    // create uniform
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    // update view projection
    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

impl Default for CameraUniform {
    // default uniform
    fn default() -> Self {
        Self::new()
    }
}
