use cgmath::{Matrix4, Rad};

// convert opengl coords to wgpu
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> =
    cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0,
    );

// view projection
pub struct Projection {
    // aspect ratio
    aspect: f32,

    // field of view
    fov: Rad<f32>,

    // z cutoffs
    z_near: f32,
    z_far:  f32,
}

impl Projection {
    // create projection
    pub fn new(width: u32, height: u32, fov: Rad<f32>, z_near: f32, z_far: f32) -> Self {
        Self {
            z_near,
            z_far,
            fov,
            aspect: width as f32 / height as f32,
        }
    }

    // resize projection
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    // calculate matrix
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(self.fov, self.aspect, self.z_near, self.z_far)
    }
}
