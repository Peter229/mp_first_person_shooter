use std::collections::HashMap;

use crate::collision;

pub enum CollisionShapes {
    Sphere(collision::Sphere, Option<glam::f32::Vec3>, u32),
    Capsule(collision::Capsule, Option<glam::f32::Vec3>, u32),
    TriangleSoup(collision::Triangle, u32),
}

pub struct CollisionWorld {
    collision_shapes: Vec<CollisionShapes>,
}

impl CollisionWorld {

    pub fn new() -> Self {

        Self { collision_shapes: Vec::new() }
    }

    pub fn add_shape(&mut self, collision_shape: CollisionShapes, layer: u32) {

        self.collision_shapes.push(collision_shape);
    }
}