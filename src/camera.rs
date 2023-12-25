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

        let view = glam::f32::Mat4::look_at_lh(self.eye, self.target, self.up);
        let projection = glam::f32::Mat4::perspective_lh(self.fov_y, self.aspect_ratio, self.z_near, self.z_far);

        return projection * view;
    }
}