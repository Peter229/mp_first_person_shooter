
//Collision detection methods
//Sphere vs Sphere
//Sphere vs Capsule

#[derive(Debug, Copy, Clone)]
pub struct CollisionPacket {
    pub collided: bool,
    pub position: glam::f32::Vec3,
    pub normal: glam::f32::Vec3,
    pub penetration: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    center: glam::f32::Vec3,
    radius: f32,
}

impl Sphere {
    
    pub fn new(center: glam::f32::Vec3, radius: f32) -> Self {

        Self { center, radius }
    }

    pub fn vs_sphere(&self, other: &Sphere) -> CollisionPacket {

        let distance_between_spheres = (self.center - other.center).length();
        let penetration = (self.radius + other.radius) - distance_between_spheres;
        let collided = distance_between_spheres <= (self.radius - other.radius).abs();
        let normal = (self.center - other.center).normalize_or_zero();
        let position = (self.center + normal * penetration) - (normal * self.radius);

        CollisionPacket { collided, position, normal, penetration }
    }

    pub fn vs_capsule(&self, other: &Capsule) -> CollisionPacket {

        let other_normal = (other.tip - other.base).normalize_or_zero();
        let other_line_end_offset = other_normal * other.radius;
        let other_a = other.base + other_line_end_offset;
        let other_b = other.tip - other_line_end_offset;

        let best_b = closest_point_on_line(&other_a, &other_b, &self.center);

        self.vs_sphere(&Sphere::new(best_b, other.get_radius()))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Capsule {
    base: glam::f32::Vec3,
    tip: glam::f32::Vec3,
    radius: f32,
}

impl Capsule {

    pub fn new(base: glam::f32::Vec3, tip: glam::f32::Vec3, radius: f32) -> Self {

        Self { base, tip, radius }
    }

    pub fn vs_sphere(&self, other: &Sphere) -> CollisionPacket {

        other.vs_capsule(self)
    }

    pub fn vs_capsule(&self, other: &Capsule) -> CollisionPacket {

        let self_normal = (self.tip - self.base).normalize_or_zero();
        let self_line_end_offset = self_normal * self.radius;
        let self_a = self.base + self_line_end_offset;
        let self_b = self.tip - self_line_end_offset;

        let other_normal = (other.tip - other.base).normalize_or_zero();
        let other_line_end_offset = other_normal * other.radius;
        let other_a = other.base + other_line_end_offset;
        let other_b = other.tip - other_line_end_offset;
        
        let v0 = other_a - self_a;
        let v1 = other_b - self_a;
        let v2 = other_a - self_b;
        let v3 = other_b - self_b;

        let d0 = v0.dot(v0);
        let d1 = v1.dot(v1);
        let d2 = v2.dot(v2);
        let d3 = v3.dot(v3);

        let best_b = if d2 < d0 || d2 < d1 || d3 < d0 || d3 < d1 {
            closest_point_on_line(&other_b, &other_a, &self_b)
        }
        else {
            closest_point_on_line(&other_b, &other_a, &self_a)
        };

        let best_a = closest_point_on_line(&self_a, &self_b, &best_b);

        let normal = (best_a - best_b).normalize();
        let len = (best_a - best_b).length();
        let penetration = self.radius + other.radius - len;
        let collided = penetration > 0.0;

        CollisionPacket { collided, position: glam::f32::Vec3::ZERO, normal, penetration }
    }

    pub fn get_radius(&self) -> f32 {

        self.radius
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    vertex_0: glam::f32::Vec3,
    vertex_1: glam::f32::Vec3,
    vertex_2: glam::f32::Vec3,
}

impl Triangle {

    pub fn new(vertex_0: glam::f32::Vec3, vertex_1: glam::f32::Vec3, vertex_2: glam::f32::Vec3) -> Self {

        Self { vertex_0, vertex_1, vertex_2 }
    }
}

#[derive(Debug, Clone)]
pub struct TriangleSoup {
    triangles: Vec<Triangle>,
}

impl TriangleSoup {

    pub fn new(triangles: Vec<Triangle>) -> Self {

        Self { triangles }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    start: glam::f32::Vec3,
    direction: glam::f32::Vec3,
}

impl Ray {

    pub fn new(start: glam::f32::Vec3, direction: glam::f32::Vec3) -> Self {

        Self { start, direction }
    }
}

pub fn closest_point_on_line(a: &glam::f32::Vec3, b: &glam::f32::Vec3, point: &glam::f32::Vec3) -> glam::f32::Vec3 {

    let ab = *b - *a;
    let t = (*point - *a).dot(ab) / ab.dot(ab);
    return (*a + (t.max(0.0).min(1.0)) * ab);
}