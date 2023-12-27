use std::f32::EPSILON;

use crate::render_commands;

//Collision detection methods
//Will probably need oct tree or quad tree for performance
//Add enum variant so can generic collide method
//Should probably automate collison and use layers to decided what should and should not collide
//If laggy after spatial tree optimazation don't return collision packet as mem alloc every collision call, instead pass reference
//M = Moving supported from row, column is stationary
//             | Sphere | Capsule | Triangle | TriangleSoup | Ray | AABB | Cylinder
//Sphere       | Yes    | Yes     | Yes      | Yes          |     |      |
//Capsule      | Yes    | Yes     |          |              |     |      |
//Triangle     | Yes    |         |          |              | Yes |      |
//TriangleSoup | Yes    |         |          |              | Yes |      |
//Ray          | Yes    |         |          |              |     |      |
//AABB         |        |         |          |              |     |      |
//Cylinder     |        |         |          |              |     |      |

//Make each type return a transform for easy debug rendering 

#[derive(Debug, Copy, Clone)]
pub struct CollisionPacket {
    pub collided: bool,
    pub position: glam::f32::Vec3,
    pub normal: glam::f32::Vec3,
    pub penetration_or_time: f32,
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
        let penetration_or_time = (self.radius + other.radius) - distance_between_spheres;
        let collided = distance_between_spheres <= (self.radius + other.radius).abs();
        let normal = (self.center - other.center).normalize_or_zero();
        let position = (self.center + normal * penetration_or_time) - (normal * self.radius);

        CollisionPacket { collided, position, normal, penetration_or_time }
    }

    pub fn vs_capsule(&self, other: &Capsule) -> CollisionPacket {

        let other_normal = (other.tip - other.base).normalize_or_zero();
        let other_line_end_offset = other_normal * other.radius;
        let other_a = other.base + other_line_end_offset;
        let other_b = other.tip - other_line_end_offset;

        let best_b = closest_point_on_line(&other_a, &other_b, &self.center);

        self.vs_sphere(&Sphere::new(best_b, other.radius))
    }

    pub fn vs_triangle(&self, triangle: &Triangle) -> CollisionPacket {

        let triangle_normal = (triangle.vertex_1 - triangle.vertex_0).cross(triangle.vertex_2 - triangle.vertex_0).normalize_or_zero();
        let distance = (self.center - triangle.vertex_0).dot(triangle_normal);
        //This is sphere vs plane collision check
        if distance < -self.radius || distance > self.radius {
            return CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: triangle_normal, penetration_or_time: distance }
        }

        //Closest point on plane to sphere
        let point_0 = self.center - triangle_normal * distance;

        let c0 = (point_0 - triangle.vertex_0).cross(triangle.vertex_1 - triangle.vertex_0);
        let c1 = (point_0 - triangle.vertex_1).cross(triangle.vertex_2 - triangle.vertex_1);
        let c2 = (point_0 - triangle.vertex_2).cross(triangle.vertex_0 - triangle.vertex_2);

        let inside = c0.dot(triangle_normal) <= 0.0 && c1.dot(triangle_normal) <= 0.0 && c2.dot(triangle_normal) <= 0.0;

        let sq_radius = self.radius * self.radius;

        let point_1 = closest_point_on_line(&triangle.vertex_0, &triangle.vertex_1, &self.center);
        let v1 = self.center - point_1;
        let sq_dist_1 = v1.dot(v1);
        let mut intersects = sq_dist_1 < sq_radius;

        let point_2 = closest_point_on_line(&triangle.vertex_1, &triangle.vertex_2, &self.center);
        let v2 = self.center - point_2;
        let sq_dist_2 = v2.dot(v2);
        intersects = sq_dist_2 < sq_radius;

        let point_3 = closest_point_on_line(&triangle.vertex_2, &triangle.vertex_0, &self.center);
        let v3 = self.center - point_3;
        let sq_dist_3 = v3.dot(v3);
        intersects |= sq_dist_3 < sq_radius;

        let mut normal = glam::f32::Vec3::Y;
        let mut penetration_or_time = 0.0;
        let mut best_point = point_0;

        if intersects || inside {
            
            if inside {
                normal = self.center - point_0;
            }
            else {

                let mut d = self.center - point_1;
                let mut sq_best_distance = d.dot(d);
                best_point = point_1;
                normal = d;

                d = self.center - point_2;
                let mut sq_dist = d.dot(d);
                if sq_dist < sq_best_distance {
                    sq_best_distance = sq_dist;
                    best_point = point_2;
                    normal = d;
                }

                d = self.center - point_3;
                sq_dist = d.dot(d);
                if sq_dist < sq_best_distance {
                    //Commented out to remove warning
                    //sq_best_distance = sq_dist;
                    best_point = point_3;
                    normal = d;
                }
            }

            penetration_or_time = self.radius - normal.length();
            normal = normal.normalize_or_zero();
        }

        CollisionPacket { collided: intersects || inside, position: best_point, normal, penetration_or_time }
    }

    pub fn vs_triangle_soup(&self, triangle_soup: &TriangleSoup) -> CollisionPacket {

        triangle_soup.vs_sphere(self)
    }

    pub fn set_center(&mut self, center: glam::f32::Vec3) {

        self.center = center;
    }

    pub fn get_transform(&self) -> glam::f32::Mat4 {

        glam::f32::Mat4::from_scale_rotation_translation(glam::f32::Vec3::new(self.radius, self.radius, self.radius), glam::f32::Quat::IDENTITY, self.center)
    }

    pub fn render(&self, render_commands: &mut Vec<render_commands::RenderCommands>) {

        render_commands.push(render_commands::RenderCommands::Model(self.get_transform(), "sphere".to_string(), "debug".to_string()));
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
        let penetration_or_time = self.radius + other.radius - len;
        let collided = penetration_or_time > 0.0;
        let position = best_b + normal * penetration_or_time;

        CollisionPacket { collided, position, normal, penetration_or_time }
    }

    pub fn get_radius(&self) -> f32 {

        self.radius
    }

    pub fn set_center(&mut self, center: glam::f32::Vec3) {

        let current_center = (self.base + self.tip) / 2.0;
        let translation = center - current_center; 
        self.base += translation;
        self.tip += translation;
    }

    pub fn render(&self, render_commands: &mut Vec<render_commands::RenderCommands>) {

        let up = (self.tip - self.base).normalize_or_zero();
        let line_end_offset = up * self.radius;
        let base_sphere_center = self.base + line_end_offset;
        let tip_sphere_center = self.tip - line_end_offset;
        let center = (self.tip + self.base) / 2.0;
        let rotation = glam::f32::Quat::from_rotation_arc(glam::f32::Vec3::Y, up);

        let base_sphere = glam::f32::Mat4::from_scale_rotation_translation(glam::f32::Vec3::new(self.radius, self.radius, self.radius), rotation, base_sphere_center);
        render_commands.push(render_commands::RenderCommands::Model(base_sphere, "sphere".to_string(), "debug".to_string()));

        let tip_sphere = glam::f32::Mat4::from_scale_rotation_translation(glam::f32::Vec3::new(self.radius, self.radius, self.radius), rotation, tip_sphere_center);
        render_commands.push(render_commands::RenderCommands::Model(tip_sphere, "sphere".to_string(), "debug".to_string()));

        let cylinder = glam::f32::Mat4::from_scale_rotation_translation(glam::f32::Vec3::new(self.radius, base_sphere_center.distance(tip_sphere_center) / 2.0, self.radius), rotation, center);
        render_commands.push(render_commands::RenderCommands::Model(cylinder, "cylinder".to_string(), "debug".to_string()));
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub vertex_0: glam::f32::Vec3,
    pub vertex_1: glam::f32::Vec3,
    pub vertex_2: glam::f32::Vec3,
}

impl Triangle {

    pub fn new(vertex_0: glam::f32::Vec3, vertex_1: glam::f32::Vec3, vertex_2: glam::f32::Vec3) -> Self {

        Self { vertex_0, vertex_1, vertex_2 }
    }

    pub fn vs_sphere(&self, sphere: &Sphere) -> CollisionPacket {

        sphere.vs_triangle(self)
    }

    pub fn vs_ray(&self, ray: &Ray) -> CollisionPacket {
        
        ray.vs_triangle(self)
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

    pub fn vs_sphere(&self, sphere: &Sphere) -> CollisionPacket {

        let mut best_collision_packet = CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: f32::MAX };

        for i in 0..self.triangles.len() {

            let collision_packet = sphere.vs_triangle(&self.triangles[i]);
            if collision_packet.collided {
                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    best_collision_packet = collision_packet;
                }
            }
        }

        best_collision_packet
    }

    pub fn vs_ray(&self, ray: &Ray) -> CollisionPacket {

        let mut best_collision_packet = CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: f32::MAX };

        for i in 0..self.triangles.len() {

            let collision_packet = ray.vs_triangle(&self.triangles[i]);
            if collision_packet.collided {
                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    best_collision_packet = collision_packet;
                }
            }
        }

        best_collision_packet
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

    pub fn vs_triangle(&self, triangle: &Triangle) -> CollisionPacket {

        let v0v1 = triangle.vertex_1 - triangle.vertex_0;
        let v0v2 = triangle.vertex_2 - triangle.vertex_0;

        let pvec = self.direction.cross(v0v2);
        let det = v0v1.dot(pvec);

        if det.abs() < EPSILON {

            return CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: 0.0 };
        }

        let inv_det = 1.0 / det;

        let tvec = self.start - triangle.vertex_0;
        let u = tvec.dot(pvec) * inv_det;
        if u < 0.0 || u > 1.0 {

            return CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: 0.0 };
        }

        let qvec = tvec.cross(v0v1);
        let v = self.direction.dot(qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {

            CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: 0.0 };
        }

        let t = v0v2.dot(qvec) * inv_det;
        let normal = v0v1.cross(v0v2);

        CollisionPacket { collided: true, position: self.start + self.direction * t, normal , penetration_or_time: t }
    }

    pub fn vs_triangle_soup(&self, triangle_soup: &TriangleSoup) -> CollisionPacket {

        triangle_soup.vs_ray(self)
    }
}

pub fn closest_point_on_line(a: &glam::f32::Vec3, b: &glam::f32::Vec3, point: &glam::f32::Vec3) -> glam::f32::Vec3 {

    let ab = *b - *a;
    let t = (*point - *a).dot(ab) / ab.dot(ab);
    return (*a + (t.max(0.0).min(1.0)) * ab);
}