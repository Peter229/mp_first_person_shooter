use crate::player::Player;

pub struct Camera {
    eye: glam::f32::Vec3,
    target: glam::f32::Vec3,
    up: glam::f32::Vec3,
    aspect_ratio: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
}

impl Camera {

    pub fn new(eye: glam::f32::Vec3, target: glam::f32::Vec3, up: glam::f32::Vec3, aspect_ratio: f32, fov_y: f32, z_near: f32, z_far: f32) -> Self {

        Self { eye, target, up, aspect_ratio, fov_y, z_near, z_far }
    }

    pub fn build_projection_matrix(&self) -> glam::f32::Mat4 {

        let view = glam::f32::Mat4::look_at_rh(self.eye, self.target, self.up);
        let projection = glam::f32::Mat4::perspective_rh(self.fov_y, self.aspect_ratio, self.z_near, self.z_far);

        return projection * view;
    }

    pub fn set_position(&mut self, position: glam::f32::Vec3) {

        self.eye = position;
    }

    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) {

        
    }

    pub fn update_from_player(&mut self, player: &Player) {

        self.eye = *player.get_position();

        self.target = self.eye + *player.get_forward();
    }
}