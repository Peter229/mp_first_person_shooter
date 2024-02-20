use std::collections::HashMap;

use wgpu::util::DeviceExt;

use crate::gpu_types;
use crate::collision;

pub struct Model {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices_count: u32,
    textures: Vec<String>,
    collision: Option<collision::TriangleSoup>,
}

impl Model {

    pub fn new(device: &wgpu::Device, path: &str, with_collision: bool) -> Self {

        let (document, buffers, _) = gltf::import(path).unwrap();
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut texture_coordinates: Vec<[f32; 2]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();

        for mesh in document.meshes() {

            for primitive in mesh.primitives() {

                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                if let Some(iter) = reader.read_positions() {
                    for vertex_position in iter {
                        vertices.push(vertex_position);
                    }
                }

                if let Some(iter) = reader.read_indices() {
                    for index in iter.into_u32() {
                        indices.push(index);
                    }
                }

                if let Some(iter) = reader.read_tex_coords(0) {
                    for texture_coordinate in iter.into_f32() {
                        texture_coordinates.push(texture_coordinate);
                    }
                }

                if let Some(iter) = reader.read_normals() {
                    for normal in iter {
                        normals.push(normal);
                    }
                }                
            }
        }

        let mut weaved_vertices: Vec<gpu_types::Vertex> = Vec::new();

        for i in 0..vertices.len() {
            weaved_vertices.push(gpu_types::Vertex { position: vertices[i], normal: normals[i], texture_coordinates: texture_coordinates[i] });
        }

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(&weaved_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let mut textures = Vec::new();

        for texture in document.textures() {
            if texture.source().name().is_some() {
                textures.push(String::from(texture.source().name().unwrap()));
            }
        }

        let mut collision = None;

        if with_collision {
            collision = Some(Model::generate_triangle_soup(&vertices, &indices));
        }

        Self { vertex_buffer, index_buffer, indices_count: indices.len() as u32, textures, collision }
    }
    
    pub fn generate_triangle_soup(vertices: &Vec<[f32; 3]>, indices: &Vec<u32>) -> collision::TriangleSoup {

        let mut triangles = Vec::new();
        for i in (0..indices.len()).step_by(3) {

            triangles.push(collision::Triangle::new(vertices[indices[i] as usize].into(), vertices[indices[i + 1] as usize].into(), vertices[indices[i + 2] as usize].into()));
        }
        collision::TriangleSoup::new(triangles)
    }

    pub fn get_collision(&self) -> &collision::TriangleSoup {

        &self.collision.as_ref().unwrap()
    }

    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {

        &self.vertex_buffer
    }

    pub fn get_index_buffer(&self) -> &wgpu::Buffer {

        &self.index_buffer
    }

    pub fn get_indices_count(&self) -> u32 {

        self.indices_count
    }

    pub fn get_textures(&self) -> &Vec<String> {

        &self.textures
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct JointsUniform {
    pub joints: [[[f32; 4]; 4]; 64], 
}

pub struct SkeletonModel {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices_count: u32,
    textures: Vec<String>,
    skeleton: Skeleton,
    joints: JointsUniform,
    joints_uniform_buffer: wgpu::Buffer, 
    joints_uniform_bind_group: wgpu::BindGroup,
    animation_controller: AnimationController,
}

impl SkeletonModel {

    pub fn new(device: &wgpu::Device, path: &str) -> Self {

        let (document, buffers, _) = gltf::import(path).unwrap();
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut texture_coordinates: Vec<[f32; 2]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        //Skeleton mesh only
        let mut weights: Vec<[f32; 4]> = Vec::new();
        let mut joints: Vec<[i32; 4]> = Vec::new();

        for mesh in document.meshes() {

            for primitive in mesh.primitives() {

                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                if let Some(iter) = reader.read_positions() {
                    for vertex_position in iter {
                        vertices.push(vertex_position);
                    }
                }

                if let Some(iter) = reader.read_indices() {
                    for index in iter.into_u32() {
                        indices.push(index);
                    }
                }

                if let Some(iter) = reader.read_tex_coords(0) {
                    for texture_coordinate in iter.into_f32() {
                        texture_coordinates.push(texture_coordinate);
                    }
                }

                if let Some(iter) = reader.read_normals() {
                    for normal in iter {
                        normals.push(normal);
                    }
                }

                //Skeleton mesh only
                if let Some(iter) = reader.read_weights(0) {
                    for weight in iter.into_f32() {
                        weights.push(weight);
                    }
                }

                if let Some(iter) = reader.read_joints(0) {
                    for joint in iter.into_u16() {
                        joints.push([joint[0] as i32, joint[1] as i32, joint[2] as i32, joint[3] as i32]);
                    }
                }
            }
        }

        let mut weaved_vertices: Vec<gpu_types::SkeletonVertex> = Vec::new();

        for i in 0..vertices.len() {
            weaved_vertices.push(gpu_types::SkeletonVertex { position: vertices[i], normal: normals[i], texture_coordinates: texture_coordinates[i], weight: weights[i], joint: joints[i] });
        }

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(&weaved_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let mut textures = Vec::new();

        for texture in document.textures() {
            if texture.source().name().is_some() {
                textures.push(String::from(texture.source().name().unwrap()));
            }
        }

        let mut skeletons = Vec::new();

        for skin in document.skins() {

            let mut inverse_bind_matrices: Vec<glam::f32::Mat4> = Vec::new();

            let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(iter) = reader.read_inverse_bind_matrices() {
                for inverse_bind_matrix in iter {
                    inverse_bind_matrices.push(glam::f32::Mat4::from_cols_array_2d(&inverse_bind_matrix));
                }
            }

            skeletons.push(Skeleton::new(&skin.joints().nth(0).unwrap(), inverse_bind_matrices));
        }

        if skeletons.len() > 1 {
            panic!("More than one skeleton, need to add compatibility for this");
        }

        let mut animations = Vec::new();

        for animation in document.animations() {

            let animation_name = animation.name().unwrap_or("None");

            animations.push(animation_name.to_string());

            for channel in animation.channels() {

                let bone = skeletons[0].get_bone_by_id(channel.target().node().index()).unwrap();

                let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));

                if let Some(gltf::animation::util::ReadOutputs::Translations(iter)) = reader.read_outputs() {
                    for translation in iter {
                        if bone.animations_translation.contains_key(animation_name) {
                            bone.animations_translation.get_mut(animation_name).unwrap().push(glam::f32::Vec3::from(translation));
                        }
                        else {
                            bone.animations_translation.insert(animation_name.to_string(), vec![glam::f32::Vec3::from(translation)]);
                        }
                    }
                }

                if let Some(gltf::animation::util::ReadOutputs::Rotations(iter)) = reader.read_outputs() {
                    for rotation in iter.into_f32() {
                        if bone.animations_rotation.contains_key(animation_name) {
                            bone.animations_rotation.get_mut(animation_name).unwrap().push(glam::f32::Quat::from_array(rotation));
                        }
                        else {
                            bone.animations_rotation.insert(animation_name.to_string(), vec![glam::f32::Quat::from_array(rotation)]);
                        }
                    }
                }

                if let Some(gltf::animation::util::ReadOutputs::Scales(iter)) = reader.read_outputs() {
                    for scale in iter {
                        if bone.animations_scale.contains_key(animation_name) {
                            bone.animations_scale.get_mut(animation_name).unwrap().push(glam::f32::Vec3::from(scale));
                        }
                        else {
                            bone.animations_scale.insert(animation_name.to_string(), vec![glam::f32::Vec3::from(scale)]);
                        }
                    }
                }
            }
        }

        let joints = JointsUniform { joints: [glam::f32::Mat4::IDENTITY.to_cols_array_2d(); 64] };

        let joints_uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Joint Uniform Buffer"),
                contents: bytemuck::cast_slice(&[joints]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let joint_uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Joint uniform bind group layout"),
        });

        let joints_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &joint_uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: joints_uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("Joint uniform bind group"),
        });

        let animation_controller = AnimationController::new(animations);

        Self { vertex_buffer, index_buffer, indices_count: indices.len() as u32, textures, skeleton: skeletons[0].clone(), joints, joints_uniform_buffer, joints_uniform_bind_group, animation_controller }
    }

    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {

        &self.vertex_buffer
    }

    pub fn get_index_buffer(&self) -> &wgpu::Buffer {

        &self.index_buffer
    }

    pub fn get_indices_count(&self) -> u32 {

        self.indices_count
    }

    pub fn update_skeleton(&mut self, time: f32) {

        let mut joints_temp = Vec::new();

        self.animation_controller.update_time(time * 20.0);

        self.skeleton.set_joints_to_pose(&mut joints_temp, &self.animation_controller.get_current_animation(), self.animation_controller.time);

        for i in 0..self.skeleton.inverse_bind_matrices.len() {

            self.joints.joints[i] = (joints_temp[i] * self.skeleton.inverse_bind_matrices[i]).to_cols_array_2d();
        }
    }

    pub fn write_skeleton_buffer(&self, queue: &wgpu::Queue) {

        queue.write_buffer(&self.joints_uniform_buffer, 0, bytemuck::cast_slice(&[self.joints]));
    }

    pub fn get_joints_bind_group(&self) -> &wgpu::BindGroup {

        &self.joints_uniform_bind_group
    }

    pub fn get_animation_controller(&self) -> &AnimationController {

        &self.animation_controller
    }

    pub fn get_mut_animation_controller(&mut self) -> &mut AnimationController {

        &mut self.animation_controller
    }
}

#[derive(Debug, Clone)]
pub struct Skeleton {

    pub root_bone: Bone,
    pub inverse_bind_matrices: Vec<glam::f32::Mat4>,
}

impl Skeleton {

    pub fn new(root_bone: &gltf::scene::Node, inverse_bind_matrices: Vec<glam::f32::Mat4>) -> Self {

        Self { root_bone: Bone::new(root_bone), inverse_bind_matrices }
    }

    pub fn visit_all_bones(&self) {

        self.root_bone.visit();
    }

    pub fn get_bone_by_id(&mut self, id: usize) -> Option<&mut Bone> {

        self.root_bone.get_bone_by_id(id)
    }

    pub fn set_joints_to_pose(&self, joints_temp: &mut Vec<glam::f32::Mat4>, name: &str, time: f32) {

        self.root_bone.set_joints_to_pose(joints_temp, glam::f32::Mat4::IDENTITY, name, time, 0);
    }
}

#[derive(Debug, Clone)]
pub struct Bone {

    pub child_bones: Vec<Bone>,
    pub transform: glam::f32::Mat4,
    pub name: String,
    pub id: usize,
    pub animations_translation: HashMap<String, Vec<glam::f32::Vec3>>,
    pub animations_rotation: HashMap<String, Vec<glam::f32::Quat>>,
    pub animations_scale: HashMap<String, Vec<glam::f32::Vec3>>,
}

impl Bone {

    pub fn new(bone: &gltf::scene::Node) -> Self {

        let mut child_bones = Vec::new();

        for child in bone.children() {

            child_bones.push(Bone::new(&child));
        }

        Self { child_bones, transform: glam::f32::Mat4::from_cols_array_2d(&bone.transform().matrix()), name: bone.name().unwrap_or("None").to_string(), id: bone.index(), animations_translation: HashMap::new(), animations_rotation: HashMap::new(), animations_scale: HashMap::new() }
    }

    pub fn visit(&self) {

        println!("{}", self.name);

        for i in 0..self.child_bones.len() {
            
            self.child_bones[i].visit();
        } 
    }

    pub fn get_bone_by_id(&mut self, id: usize) -> Option<&mut Bone> {

        if self.id == id {
            return Some(self);
        }
        else {
            for child in &mut self.child_bones {

                let val = child.get_bone_by_id(id);
                if val.is_some() {
                    return val;
                }
            }
        }

        None
    }

    pub fn set_joints_to_pose(&self, joints_temp: &mut Vec<glam::f32::Mat4>, parent_matrix: glam::f32::Mat4, name: &str, time: f32, depth: i32) {

        let index = (time.floor() as u32 % self.animations_translation.get(name).unwrap().len() as u32) as usize;

        let next_index = (index + 1).min(self.animations_translation.get(name).unwrap().len() - 1);

        let lerp_amount = time - time.floor();

        let t = self.animations_translation.get(name).unwrap()[index].lerp(self.animations_translation.get(name).unwrap()[next_index], lerp_amount);
        let r = self.animations_rotation.get(name).unwrap()[index].slerp(self.animations_rotation.get(name).unwrap()[next_index], lerp_amount);
        let s = self.animations_scale.get(name).unwrap()[index].lerp(self.animations_scale.get(name).unwrap()[next_index], lerp_amount);

        let mat = parent_matrix * glam::f32::Mat4::from_scale_rotation_translation(s, r, t);

        joints_temp.push(mat);
        for bone in &self.child_bones {
            bone.set_joints_to_pose(joints_temp, mat, name, time, depth + 1)
        }
    }
}

pub struct AnimationController {

    current_animation: String,
    animations: Vec<String>,
    time: f32,
}

impl AnimationController {

    pub fn new(animations: Vec<String>) -> Self {

        let current_animation = animations.get(0).unwrap_or(&"".to_string()).to_string();

        Self { current_animation, animations, time: 0.0 }
    }

    pub fn get_animations(&self) -> &Vec<String> {

        &self.animations
    }

    pub fn get_current_animation(&self) -> &String {

        &self.current_animation
    }

    pub fn set_current_animation(&mut self, name: &str) {

        if name.to_string() == self.current_animation {
            return;
        }

        self.time = 0.0;
        self.current_animation = name.to_string();
    }

    pub fn update_time(&mut self, delta: f32) {


        self.time += delta;
    }
}