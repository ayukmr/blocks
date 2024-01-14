use crate::camera::Camera;

use cgmath::{Vector3, Rad, InnerSpace};

use winit::event::{WindowEvent, KeyEvent};
use winit::keyboard::KeyCode;
use std::f32::consts::FRAC_PI_2;

// camera controller
pub struct CameraController {
    // settings
    initial: f32,
    speed:   f32,
    sens:    f32,

    // rotation
    pub rot_h: f32,
    pub rot_v: f32,

    // z movement
    forward:  f32,
    backward: f32,

    // x movement
    left:  f32,
    right: f32,

    // y movement
    up:   f32,
    down: f32,

    // w sprinting
    last_w: instant::Instant,
}

impl CameraController {
    // create controller
    pub fn new(speed: f32, sens: f32) -> Self {
        Self {
            initial: speed,
            speed,
            sens,

            rot_h: 0.0,
            rot_v: 0.0,

            forward:  0.0,
            backward: 0.0,

            left:  0.0,
            right: 0.0,

            up:   0.0,
            down: 0.0,

            last_w: instant::Instant::now(),
        }
    }

    // handle window event
    pub fn event(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::KeyboardInput {
            event: KeyEvent {
                state,
                physical_key: winit::keyboard::PhysicalKey::Code(keycode),
                ..
            },
            ..
        } = event {
            // stop or start movement
            let amount = if state.is_pressed() { 1.0 } else { 0.0 };

            match keycode {
                KeyCode::KeyW => {
                    if state.is_pressed() && self.forward == 0.0 {
                        let now = instant::Instant::now();

                        if now - self.last_w < instant::Duration::from_millis(300) {
                            // w sprinting
                            self.speed *= 4.0;
                        }

                        self.last_w = now;
                    }

                    if !state.is_pressed() {
                        self.speed = self.initial;
                    }

                    self.forward = amount;
                    true
                }

                KeyCode::KeyS => {
                    self.backward = amount;
                    true
                }

                KeyCode::KeyA => {
                    self.left = amount;
                    true
                }

                KeyCode::KeyD => {
                    self.right = amount;
                    true
                }

                KeyCode::Space => {
                    self.up = amount;
                    true
                }

                KeyCode::ShiftLeft => {
                    self.down = amount;
                    true
                }

                _ => false,
            }
        } else {
            false
        }
    }

    // handle mouse event
    pub fn mouse(&mut self, dx: f64, dy: f64) {
        self.rot_h = dx as f32;
        self.rot_v = dy as f32;
    }

    // update camera
    pub fn update_camera(&mut self, camera: &mut Camera, dt: instant::Duration) {
        let dt = dt.as_secs_f32();
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();

        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right   = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        // z movement
        camera.pos += forward * (self.forward - self.backward) * self.speed * dt;

        // x movement
        camera.pos += right * (self.right - self.left) * self.speed * dt;

        // y movement
        camera.pos.y += (self.up - self.down) * self.speed * dt;

        // view direction
        camera.yaw   += Rad(self.rot_h) * self.sens * dt;
        camera.pitch += Rad(-self.rot_v) * self.sens * dt;

        self.rot_h = 0.0;
        self.rot_v = 0.0;

        // clamp to π/2 rad
        camera.pitch = Rad(camera.pitch.0.clamp(-FRAC_PI_2 + 0.001, FRAC_PI_2 - 0.001));
    }
}
