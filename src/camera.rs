use std::time::Duration;
use wgpu_sandbox::prelude::winit::event::{
    ElementState, MouseButton, MouseScrollDelta, WindowEvent,
};

use glam::{
    f32::{Mat4, Vec3},
    vec3,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub eye: Vec3,
    pub fov: f32,
    pub target: Vec3,
}

impl Camera {
    pub fn new(eye: Vec3, target: Vec3, fov: f32) -> Self {
        Self { eye, target, fov }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: vec3(0.0, 0.0, 2.0),
            target: Vec3::ZERO,
            fov: 89.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraController {
    pub camera: Camera,
    screen_size: (f32, f32),
    is_middle_button_hold: bool,
    last_cursor: (f64, f64),
    rotation: (f32, f32),
    zoom: f32,
}

impl CameraController {
    const ROTATION_SPEED: f32 = 200.0;
    const ZOOM_SPEED: f32 = 100.0;

    pub fn new(camera: Camera, screen_size: (f32, f32)) -> Self {
        Self {
            camera,
            screen_size,
            is_middle_button_hold: false,
            last_cursor: (0.0, 0.0),
            rotation: (0.0, 0.0),
            zoom: 0.0,
        }
    }

    pub fn handle_events(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
                ..
            } => self.is_middle_button_hold = true,
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                ..
            } => self.is_middle_button_hold = false,
            WindowEvent::CursorMoved { position, .. } => {
                let (dx, dy) = (
                    (self.last_cursor.0 - position.x) as f32 / self.screen_size.0,
                    (self.last_cursor.1 - position.y) as f32 / self.screen_size.1,
                );
                self.rotation = (-std::f32::consts::FRAC_PI_2 * dy, std::f32::consts::PI * dx);
                self.last_cursor = (position.x, position.y);
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, dy) => self.zoom = *dy,
                _ => (),
            },
            _ => (),
        }
    }

    pub fn update(&mut self, dt: Duration) {
        let dir = (self.camera.target - self.camera.eye).normalize();

        if self.is_middle_button_hold {
            let rotx = Mat4::from_axis_angle(
                Vec3::Y.cross(dir),
                self.rotation.0 * Self::ROTATION_SPEED * dt.as_secs_f32(),
            );
            let roty =
                Mat4::from_rotation_y(self.rotation.1 * Self::ROTATION_SPEED * dt.as_secs_f32());

            let eye = rotx.transform_vector3(self.camera.eye);
            let eye = roty.transform_vector3(eye);

            if (eye - self.camera.target).normalize().dot(Vec3::Y).abs() < 0.95 {
                self.camera.eye = eye;
            }
        }

        if self.zoom != 0.0 {
            self.camera.eye = Mat4::from_translation(
                dir * Self::ZOOM_SPEED * dt.as_secs_f32() * self.zoom as f32,
            )
            .transform_point3(self.camera.eye);
        }

        // reset properties
        self.rotation = (0.0, 0.0);
        self.zoom = 0.0;
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            screen_size: (1280.0, 720.0),
            rotation: (0.0, 0.0),
            last_cursor: (0.0, 0.0),
            zoom: 0.0,
            is_middle_button_hold: false,
        }
    }
}
