use wgpu::util::DeviceExt;

use crate::gpu_types;

pub struct Model {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices_count: u32,
    material: String,
}

impl Model {

    pub fn new(device: &wgpu::Device, path: &str) -> Self {

        let (document, buffers, _) = gltf::import(path).unwrap();

        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut texture_coordinates: Vec<[f32; 2]> = Vec::new();

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
            }
        }

        let mut weaved_vertices: Vec<gpu_types::Vertex> = Vec::new();

        for i in 0..vertices.len() {
            weaved_vertices.push(gpu_types::Vertex { position: vertices[i], texture_coordinates: texture_coordinates[i] });
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

        Self { vertex_buffer, index_buffer, indices_count: indices.len() as u32, material: String::from("None") }
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
}