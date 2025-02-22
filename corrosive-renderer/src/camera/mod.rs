use nalgebra::{Isometry3, Matrix4, Perspective3, Point3, Vector3};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32, // in degrees
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        // Create the view matrix using a right-handed coordinate system.
        let view = Isometry3::look_at_rh(&self.eye, &self.target, &self.up).to_homogeneous();

        // Convert field-of-view from degrees to radians and create the projection matrix.
        let proj = Perspective3::new(self.aspect, self.fovy.to_radians(), self.znear, self.zfar)
            .to_homogeneous();

        // Combine the matrices.
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use nalgebra::SquareMatrix;
        Self {
            view_proj: nalgebra::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub(crate) fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub(crate) fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        // Calculate forward vector from eye to target.
        let forward: Vector3<f32> = camera.target - camera.eye;
        let forward_mag: f32 = forward.norm();

        // Normalize the forward vector.
        let mut forward_norm: Vector3<f32> = forward.normalize();

        // Move camera along the forward direction.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        // Calculate the right vector via cross product with up vector.
        let right: Vector3<f32> = forward_norm.cross(&camera.up);

        // Recalculate forward vector and magnitude after movement.
        let forward: Vector3<f32> = camera.target - camera.eye;
        let forward_mag: f32 = forward.norm();

        if self.is_right_pressed {
            // Shift the camera right while keeping the same distance to the target.
            let new_direction = (forward + right * self.speed).normalize();
            camera.eye = camera.target - new_direction * forward_mag;
        }
        if self.is_left_pressed {
            let new_direction = (forward - right * self.speed).normalize();
            camera.eye = camera.target - new_direction * forward_mag;
        }
    }
}
