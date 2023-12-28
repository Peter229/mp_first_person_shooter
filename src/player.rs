use crate::{collision, input::{Inputs, MOUSE_SENSITIVITY}};

pub struct Player {
    position: glam::f32::Vec3,
    velocity: glam::f32::Vec3,
    forward: glam::f32::Vec3,
    right: glam::f32::Vec3,
    yaw: f32,
    pitch: f32,
}

impl Player {
    
    pub fn new(position: glam::f32::Vec3) -> Self {

        Self { position, velocity: glam::f32::Vec3::ZERO, forward: glam::f32::Vec3::Z, right: glam::f32::Vec3::X, yaw: 0.0, pitch: 0.0 }
    }

    pub fn translate(&mut self, translation: glam::f32::Vec3) {

        self.position += translation;
    }

    pub fn translate_relative(&mut self, translation: glam::f32::Vec3) {

        self.position += translation.z * self.forward;
        self.position += translation.x * glam::f32::Vec3::Y.cross(self.forward).normalize_or_zero();
    }

    pub fn get_position(&self) -> glam::f32::Vec3 {

        self.position
    }

    pub fn input(&mut self, inputs: &mut Inputs) {
        
        let mouse_motion = inputs.get_mouse_motion();

        self.yaw -= (mouse_motion[0] * MOUSE_SENSITIVITY) % 360.0_f32.to_radians();

        self.pitch -= (mouse_motion[1] * MOUSE_SENSITIVITY).clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

        self.forward = glam::f32::Vec3::new(self.pitch.cos() * self.yaw.cos(), self.pitch.sin(), self.pitch.cos() * self.yaw.sin()).normalize_or_zero();
    }

    pub fn get_camera_transform(&self) -> glam::f32::Mat4 {

        glam::f32::Mat4::from_rotation_translation(glam::f32::Quat::from_rotation_y(self.yaw), self.position)
    }

    pub fn get_yaw(&self) -> f32 {

        self.yaw
    }

    pub fn get_pitch(&self) -> f32 {

        self.pitch
    }

    pub fn get_forward(&self) -> &glam::f32::Vec3 {

        &self.forward
    }
}