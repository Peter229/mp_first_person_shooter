use crate::collision;

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

    pub fn get_position(&self) -> glam::f32::Vec3 {

        self.position
    }
}