use std::collections::HashMap;

use crate::model;
use crate::texture;

pub struct ResourceManager {
    models: HashMap<String, model::Model>,
    textures: HashMap<String, texture::Texture>,
}

impl ResourceManager {

    pub fn new() -> Self {
        
        Self { models: HashMap::new(), textures: HashMap::new() }
    }

    pub fn load_model(&mut self, device: &wgpu::Device, path: &str, with_collision: bool) -> f32 {

        let start = std::time::Instant::now();

        let name = path.split("/").last().unwrap().split(".").nth(0).unwrap();

        if self.models.contains_key(name) {

            eprintln!("Already loaded model: {} at {}", name, path);
        }
        else {

            println!("Now loading {} at path {}", name, path);
            self.models.insert(name.to_string(), model::Model::new(device, path, with_collision));
        }

        let milli_time = (start.elapsed().as_micros() as f32 / 1000.0);
        milli_time
    }

    pub fn load_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, path: &str) -> f32 {
        
        let start = std::time::Instant::now();

        let name = path.split("/").last().unwrap().split(".").nth(0).unwrap();

        if self.textures.contains_key(name) {

            eprintln!("Already loaded texture: {} at {}", name, path);
        }
        else {
            
            println!("Now loading {} at path {}", name, path);
            let diffuse_texture = texture::Texture::from_disk(&device, &queue, path, name).unwrap();
            self.textures.insert(name.to_string(), diffuse_texture);
        }

        let milli_time = (start.elapsed().as_micros() as f32 / 1000.0);
        milli_time
    }

    pub fn get_model(&self, name: &String) -> Option<&model::Model> {

        self.models.get(name)
    }

    pub fn get_texture(&self, name: &String) -> Option<&texture::Texture> {

        self.textures.get(name)
    }

    pub fn bulk_load(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> f32 {

        let start = std::time::Instant::now();
        
        //Make just scan folder, need better way of telling we should generate collision
        let things_to_load: Vec<(&str, bool)> = vec![("./assets/cube.glb", false),
            ("./assets/sphere.glb", false),
            ("./assets/capsule.glb", false),
            ("./assets/cylinder.glb", false),
            ("./assets/test_triangle.glb", true),
            ("./assets/dot_crosshair.png", false),
            ("./assets/tree.jpg", false),
            ("./assets/debug.png", false)];

        for (path, collide) in things_to_load {
            if path.contains(".glb") {
                self.load_model(device, path, collide);
            }
            else {
                self.load_texture(device, queue, path);
            }
        }

        let milli_time = (start.elapsed().as_micros() as f32 / 1000.0);
        milli_time
    }
}