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

    pub fn load_model(&mut self, device: &wgpu::Device, path: &str, name: &str) {

        if self.models.contains_key(name) {

            eprintln!("Already loaded model: {} at {}", name, path);
        }
        else {

            self.models.insert(name.to_string(), model::Model::new(device, path));
        }
    }

    pub fn load_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, path: &str, name: &str) {

        if self.textures.contains_key(name) {

            eprintln!("Already loaded texture: {} at {}", name, path);
        }
        else {
            
            let diffuse_texture = texture::Texture::from_disk(&device, &queue, path, name).unwrap();
            self.textures.insert(name.to_string(), diffuse_texture);
        }
    }

    pub fn get_model(&self, name: &String) -> Option<&model::Model> {

        self.models.get(name)
    }

    pub fn get_texture(&self, name: &String) -> Option<&texture::Texture> {

        self.textures.get(name)
    }
}