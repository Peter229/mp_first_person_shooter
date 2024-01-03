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
            if texture.name().is_some() {
                println!("Texture at {}", texture.name().unwrap());
                textures.push(String::from(texture.name().unwrap()));
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