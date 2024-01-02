use std::f32::EPSILON;

use crate::render_commands;

//Collision detection methods
//Will probably need oct tree or quad tree for performance
//Add enum variant so can generic collide method
//Should probably automate collison and use layers to decided what should and should not collide
//If laggy after spatial tree optimazation don't return collision packet as mem alloc every collision call, instead pass reference
//To do CCD expand shape by colliding shape and then just do ray vs the expanded shape
//M = Moving supported from row, column is stationary
//             | Sphere | Capsule | Triangle | TriangleSoup | Ray | AABB | Cylinder
//Sphere       | Yes    | Yes     | Yes M    | Yes M        | Yes |      |
//Capsule      | Yes    | Yes     | Yes M    | Yes M        | Yes |      |
//Triangle     | Yes M  | Yes M   |          |              | Yes |      |
//TriangleSoup | Yes M  | Yes M   |          |              | Yes |      |
//Ray          | Yes    | Yes     | Yes      | Yes          |     |      |
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
        intersects |= sq_dist_2 < sq_radius;

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

    pub fn vs_while_moving_triangle(&self, velocity: &glam::f32::Vec3, triangle: &Triangle) -> CollisionPacket {

        let ray = Ray::new(self.center, *velocity);

        let mut best_collision_packet = ray.vs_cylinder(&triangle.vertex_0, &triangle.vertex_1, self.radius); 

        let mut collision_packet = ray.vs_cylinder(&triangle.vertex_1, &triangle.vertex_2, self.radius);
        if collision_packet.collided {

            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {

                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        collision_packet = ray.vs_cylinder(&triangle.vertex_2, &triangle.vertex_0, self.radius);
        if collision_packet.collided {

            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        collision_packet = ray.vs_sphere(&Sphere::new(triangle.vertex_0, self.radius));
        if collision_packet.collided {

            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        collision_packet = ray.vs_sphere(&Sphere::new(triangle.vertex_1, self.radius));
        if collision_packet.collided {

            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        collision_packet = ray.vs_sphere(&Sphere::new(triangle.vertex_2, self.radius));
        if collision_packet.collided {

            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        //Front face
        let v1v0 = triangle.vertex_1 - triangle.vertex_0;
        let v2v0 = triangle.vertex_2 - triangle.vertex_0;
        let triangle_normal_offset = v1v0.cross(v2v0).normalize_or_zero() * self.radius;
        collision_packet = ray.vs_triangle(&Triangle::new(triangle.vertex_0 + triangle_normal_offset, triangle.vertex_1 + triangle_normal_offset, triangle.vertex_2 + triangle_normal_offset));
        if collision_packet.collided {

            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        //Back face
        collision_packet = ray.vs_triangle(&Triangle::new(triangle.vertex_0 - triangle_normal_offset, triangle.vertex_2 - triangle_normal_offset, triangle.vertex_1 - triangle_normal_offset));
        if collision_packet.collided {

            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        best_collision_packet.collided = collision_packet.penetration_or_time < 1.0;

        best_collision_packet
    }

    pub fn vs_ray(&self, ray: &Ray) -> CollisionPacket {

        ray.vs_sphere(self)
    }

    pub fn vs_triangle_soup(&self, triangle_soup: &TriangleSoup) -> CollisionPacket {

        triangle_soup.vs_sphere(self)
    }

    pub fn vs_while_moving_triangle_soup(&self, velocity: &glam::f32::Vec3, triangle_soup: &TriangleSoup) -> CollisionPacket {

        triangle_soup.vs_moving_sphere(self, velocity)
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

    pub fn vs_triangle(&self, triangle: &Triangle) -> CollisionPacket {

        let triangle_normal = ((triangle.vertex_1 - triangle.vertex_0).cross(triangle.vertex_2 - triangle.vertex_0)).normalize_or_zero();

        let capsule_normal = (self.tip - self.base).normalize_or_zero();

        let line_end_offset = capsule_normal * self.radius;
        let a = self.base + line_end_offset;
        let b = self.tip - line_end_offset;

        let mut reference_point = triangle.vertex_0;

        if capsule_normal.dot(triangle_normal) != 0.0 {

            let t = triangle_normal.dot(triangle.vertex_0 - self.base) / triangle_normal.dot(capsule_normal).abs();
            let line_plane_intersection = self.base + capsule_normal * t;

            reference_point = closest_point_on_triangle(triangle, &triangle_normal, &line_plane_intersection);
        }

        let center = closest_point_on_line(&a, &b, &reference_point);

        triangle.vs_sphere(&Sphere::new(center, self.radius))
    }

    pub fn vs_while_moving_triangle(&self, velocity: &glam::f32::Vec3, triangle: &Triangle) -> CollisionPacket {

        let ray = Ray::new((self.base + self.tip) / 2.0, *velocity);

        let self_normal = (self.tip - self.base).normalize_or_zero();

        let mut test_capsule = self.clone();
        //Test against triangle vertexes
        test_capsule.set_center(triangle.vertex_0);
        let tip_1 = test_capsule.tip;
        let base_1 = test_capsule.base;
        let mut best_collision_packet = ray.vs_capsule(&test_capsule);

        test_capsule.set_center(triangle.vertex_1);
        let tip_2 = test_capsule.tip;
        let base_2 = test_capsule.base;
        let mut collision_packet = ray.vs_capsule(&test_capsule);
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {

                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        test_capsule.set_center(triangle.vertex_2);
        let tip_3 = test_capsule.tip;
        let base_3 = test_capsule.base;
        collision_packet = ray.vs_capsule(&test_capsule);
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {

                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        //Triangle edges top
        let mut test_capsule_direction = (tip_2 - tip_1).normalize_or_zero() * self.radius;
        let point_a = tip_1 - self_normal * self.radius;
        let point_b = tip_2 - self_normal * self.radius;
        let point_c = tip_3 - self_normal * self.radius;
        let mut capsule = Capsule::new(point_b + test_capsule_direction, point_a - test_capsule_direction, self.radius);
        collision_packet = ray.vs_capsule(&capsule);
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        test_capsule_direction = (tip_3 - tip_1).normalize_or_zero() * self.radius;
        capsule = Capsule::new(point_c + test_capsule_direction, point_a - test_capsule_direction, self.radius);
        collision_packet = ray.vs_capsule(&capsule);
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        test_capsule_direction = (tip_3 - tip_2).normalize_or_zero() * self.radius;
        capsule = Capsule::new(point_c + test_capsule_direction, point_b - test_capsule_direction, self.radius);
        collision_packet = ray.vs_capsule(&capsule);
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        //Triangle edges bottom
        test_capsule_direction = (base_2 - base_1).normalize_or_zero() * self.radius;
        let point_a_base = base_1 + self_normal * self.radius;
        let point_b_base = base_2 + self_normal * self.radius;
        let point_c_base = base_3 + self_normal * self.radius;
        capsule = Capsule::new(point_b_base + test_capsule_direction, point_a_base - test_capsule_direction, self.radius);
        collision_packet = ray.vs_capsule(&capsule);
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        test_capsule_direction = (base_3 - base_1).normalize_or_zero() * self.radius;
        capsule = Capsule::new(point_c_base + test_capsule_direction, point_a_base - test_capsule_direction, self.radius);
        collision_packet = ray.vs_capsule(&capsule);
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        test_capsule_direction = (base_3 - base_2).normalize_or_zero() * self.radius;
        capsule = Capsule::new(point_c_base + test_capsule_direction, point_b_base - test_capsule_direction, self.radius);
        collision_packet = ray.vs_capsule(&Capsule::new(point_c_base + test_capsule_direction, point_b_base - test_capsule_direction, self.radius));
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        //Sides
        let triangle_normal = (triangle.vertex_1 - triangle.vertex_0).cross(triangle.vertex_2 - triangle.vertex_0).normalize_or_zero();
        let edge_1_normal = (triangle.vertex_1 - triangle.vertex_0).cross(triangle_normal).normalize_or_zero();
        collision_packet = ray.vs_triangle(&Triangle::new(point_a + edge_1_normal * self.radius, point_b + edge_1_normal * self.radius, point_a_base + edge_1_normal * self.radius));
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        collision_packet = ray.vs_triangle(&Triangle::new(point_a_base + edge_1_normal * self.radius, point_b_base + edge_1_normal * self.radius, point_b + edge_1_normal * self.radius));
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        let edge_2_normal = triangle_normal.cross(triangle.vertex_2 - triangle.vertex_0).normalize_or_zero();
        collision_packet = ray.vs_triangle(&Triangle::new(point_a + edge_2_normal * self.radius, point_c + edge_2_normal * self.radius, point_a_base + edge_2_normal * self.radius));
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        collision_packet = ray.vs_triangle(&Triangle::new(point_a_base + edge_2_normal * self.radius, point_c_base + edge_2_normal * self.radius, point_c + edge_2_normal * self.radius));
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        let edge_3_normal = (triangle.vertex_2 - triangle.vertex_1).cross(triangle_normal).normalize_or_zero();
        collision_packet = ray.vs_triangle(&Triangle::new(point_b + edge_3_normal * self.radius, point_c + edge_3_normal * self.radius, point_b_base + edge_3_normal * self.radius));
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }
        collision_packet = ray.vs_triangle(&Triangle::new(point_b_base + edge_3_normal * self.radius, point_c_base + edge_3_normal * self.radius, point_c + edge_3_normal * self.radius));
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        //Front face
        collision_packet = ray.vs_triangle(&Triangle::new(tip_1, tip_2, tip_3));
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        //Back face
        collision_packet = ray.vs_triangle(&Triangle::new(base_1, base_3, base_2));
        if collision_packet.collided {
            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        best_collision_packet.collided &= best_collision_packet.penetration_or_time.abs() < 1.0;

        best_collision_packet
    }

    pub fn vs_while_moving_triangle_soup(&self, velocity: &glam::f32::Vec3, triangle_soup: &TriangleSoup) -> CollisionPacket {

        triangle_soup.vs_moving_capsule(self, velocity)
    }

    pub fn vs_triangle_soup(&self, triangle_soup: &TriangleSoup) -> CollisionPacket {

        triangle_soup.vs_capsule(self)
    }

    pub fn vs_ray(&self, ray: &Ray) -> CollisionPacket {

        ray.vs_capsule(self)
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

    pub fn get_center(&self) -> glam::f32::Vec3 {

        (self.base + self.tip) / 2.0
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

    pub fn vs_moving_sphere(&self, sphere: &Sphere, velocity: &glam::f32::Vec3) -> CollisionPacket {

        sphere.vs_while_moving_triangle(velocity, self)
    }

    pub fn vs_moving_capsule(&self, capsule: &Capsule, velocity: &glam::f32::Vec3) -> CollisionPacket {

        capsule.vs_while_moving_triangle(velocity, self)
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

    pub fn vs_capsule(&self, capsule: &Capsule) -> CollisionPacket {

        let mut best_collision_packet = CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: f32::MAX };

        for i in 0..self.triangles.len() {

            let collision_packet = capsule.vs_triangle(&self.triangles[i]);
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

    pub fn vs_moving_sphere(&self, sphere: &Sphere, velocity: &glam::f32::Vec3) -> CollisionPacket {

        let mut best_collision_packet = CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: f32::MAX };

        for i in 0..self.triangles.len() {

            let collision_packet = sphere.vs_while_moving_triangle(velocity, &self.triangles[i]);
            if collision_packet.collided {
                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    best_collision_packet = collision_packet;
                }
            }
        }

        best_collision_packet
    }

    pub fn vs_moving_capsule(&self, capsule: &Capsule, velocity: &glam::f32::Vec3) -> CollisionPacket {

        let mut best_collision_packet = CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: f32::MAX };

        for i in 0..self.triangles.len() {

            let collision_packet = capsule.vs_while_moving_triangle(velocity, &self.triangles[i]);
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

    pub fn vs_sphere(&self, sphere: &Sphere) -> CollisionPacket {

        let co = self.start - sphere.center;

        let a = self.direction.dot(self.direction);
        let b = co.dot(self.direction);
        let c = co.dot(co) - (sphere.radius * sphere.radius);

        let discriminant = b * b - a * c;
        if discriminant < 0.0 {

            return CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: 0.0 };
        }

        let penetration_or_time = (-b - discriminant.sqrt()) / a;
        let collided = penetration_or_time > 0.0;

        let normal = co + penetration_or_time * self.direction;
        
        CollisionPacket { collided, position: self.start + self.direction * penetration_or_time, normal, penetration_or_time }
    }

    //Maybe I should make a cylinder struct
    pub fn vs_cylinder(&self, base: &glam::f32::Vec3, tip: &glam::f32::Vec3, radius: f32) -> CollisionPacket {

        let ray_direction = self.direction.normalize_or_zero();
        let cylinder_center = (*base + *tip) / 2.0;
        let mut ch = (*tip - *base).length();
        let ca = (*tip - *base) / ch;

        let oc = self.start - cylinder_center;
        ch *= 0.5;

        let card = ca.dot(ray_direction);
        let caoc = ca.dot(oc);

        let a = 1.0 - card * card;
        let b = oc.dot(ray_direction) - caoc * card;
        let c = oc.dot(oc) - caoc * caoc - radius * radius;
        let h = b * b - a * c;
        if h < 0.0 {

            return CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: 0.0 };
        }

        let t = (-b - (h.sqrt())) / a;
        let y = caoc + t * card;
        if y.abs() > ch {

            return CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: 0.0 };
        }

        let normal = (oc + t * ray_direction - ca * y).normalize();
        let collided = t > 0.0;
        let penetration_or_time = t / (self.direction).length();

        return CollisionPacket { collided, position: self.start + ray_direction * t, normal, penetration_or_time };
    }

    pub fn vs_capsule(&self, capsule: &Capsule) -> CollisionPacket {

        let capsule_length = (capsule.tip - capsule.base).normalize_or_zero();
        let self_line_end_offset = capsule_length * capsule.radius;
        let a = capsule.base + self_line_end_offset;
        let b = capsule.tip - self_line_end_offset;

        let mut best_collision_packet = self.vs_sphere(&Sphere::new(a, capsule.radius));

        let mut collision_packet = self.vs_sphere(&Sphere::new(b, capsule.radius));
        if collision_packet.collided {

            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        collision_packet = self.vs_cylinder(&capsule.base, &capsule.tip, capsule.radius);
        if collision_packet.collided {

            if best_collision_packet.collided {

                if collision_packet.penetration_or_time < best_collision_packet.penetration_or_time {
                    best_collision_packet = collision_packet;
                }
            }
            else {

                best_collision_packet = collision_packet;
            }
        }

        best_collision_packet
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

            return CollisionPacket { collided: false, position: glam::f32::Vec3::ZERO, normal: glam::f32::Vec3::Y, penetration_or_time: 0.0 };
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

pub fn closest_point_on_triangle(triangle: &Triangle, triangle_normal: &glam::f32::Vec3, point: &glam::f32::Vec3)  -> glam::f32::Vec3 {

    let c0 = (*point - triangle.vertex_0).cross(triangle.vertex_1 - triangle.vertex_0);
    let c1 = (*point - triangle.vertex_1).cross(triangle.vertex_2 - triangle.vertex_1);
    let c2 = (*point - triangle.vertex_2).cross(triangle.vertex_0 - triangle.vertex_2);
    
    let inside = c0.dot(*triangle_normal) <= 0.0 && c1.dot(*triangle_normal) <= 0.0 && c2.dot(*triangle_normal) <= 0.0;

    if inside {

        return *point;
    }

    let point_1 = closest_point_on_line(&triangle.vertex_0, &triangle.vertex_1, point);
    let v1 = *point - point_1;
    let mut sq_dist = v1.dot(v1);
    let mut best_dist = sq_dist;
    let mut closest_point = point_1;

    let point_2 = closest_point_on_line(&triangle.vertex_1, &triangle.vertex_2, point);
    let v2 = *point - point_2;
    sq_dist = v2.dot(v2);
    if sq_dist < best_dist {
        closest_point = point_2;
    }

    let point_3 = closest_point_on_line(&triangle.vertex_1, &triangle.vertex_2, point);
    let v3 = *point - point_3;
    sq_dist = v3.dot(v3);
    if sq_dist < best_dist {
        closest_point = point_3;
    }
    
    closest_point
}