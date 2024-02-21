use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::audio::WavAudioData;
use crate::console::*;
use crate::model;
use crate::texture;
use crate::audio;

pub struct ResourceManager {
    models: HashMap<String, model::Model>,
    skeleton_models: HashMap<String, model::SkeletonModel>,
    textures: HashMap<String, texture::Texture>,
    sounds: HashMap<String, WavAudioData>,
    console: Rc<RefCell<Console>>,
}

impl ResourceManager {

    pub fn new(console: Rc<RefCell<Console>>) -> Self {
        
        Self { models: HashMap::new(), skeleton_models: HashMap::new(), textures: HashMap::new(), sounds: HashMap::new(), console }
    }

    pub fn load_model(&mut self, device: &wgpu::Device, path: &str, with_collision: bool) {

        let start = std::time::Instant::now();

        let name = path.split("/").last().unwrap().split(".").nth(0).unwrap();

        if self.models.contains_key(name) {
            self.console.borrow_mut().output_to_console(&format!("Already loaded model: {} at {}", name, path));
        }
        else {
            self.console.borrow_mut().output_to_console(&format!("Now loading {} at path {}", name, path));
            self.models.insert(name.to_string(), model::Model::new(device, path, with_collision));
        }

        let milli_time = start.elapsed().as_micros() as f32 / 1000.0;
        self.console.borrow_mut().output_to_console(&format!("{} took {}ms to load", name, milli_time));
    }

    pub fn load_skeleton_model(&mut self, device: &wgpu::Device, path: &str) {

        let start = std::time::Instant::now();

        let name = path.split("/").last().unwrap().split(".").nth(0).unwrap();

        if self.skeleton_models.contains_key(name) {
            self.console.borrow_mut().output_to_console(&format!("Already loaded skeleton model: {} at {}", name, path));
        }
        else {
            self.console.borrow_mut().output_to_console(&format!("Now loading {} at path {}", name, path));
            self.skeleton_models.insert(name.to_string(), model::SkeletonModel::new(device, path));
        }

        let milli_time = start.elapsed().as_micros() as f32 / 1000.0;
        self.console.borrow_mut().output_to_console(&format!("{} took {}ms to load", name, milli_time));
    }

    pub fn load_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, path: &str) {
        
        let start = std::time::Instant::now();

        let name = path.split("/").last().unwrap().split(".").nth(0).unwrap();

        if self.textures.contains_key(name) {
            self.console.borrow_mut().output_to_console(&format!("Already loaded texture: {} at {}", name, path));
        }
        else {
            self.console.borrow_mut().output_to_console(&format!("Now loading {} at path {}", name, path));
            let diffuse_texture = texture::Texture::from_disk(&device, &queue, path, name).unwrap();
            self.textures.insert(name.to_string(), diffuse_texture);
        }

        let milli_time = start.elapsed().as_micros() as f32 / 1000.0;
        self.console.borrow_mut().output_to_console(&format!("{} took {}ms to load", name, milli_time));
    }

    pub fn load_wav(&mut self, path: &str) {

        let start = std::time::Instant::now();

        let name = path.split("/").last().unwrap().split(".").nth(0).unwrap();

        if self.textures.contains_key(name) {
            self.console.borrow_mut().output_to_console(&format!("Already loaded sound: {} at {}", name, path));
        }
        else {
            self.console.borrow_mut().output_to_console(&format!("Now loading {} at path {}", name, path));
            let wav_audio_data = audio::WavAudioData::new(path);
            self.sounds.insert(name.to_string(), wav_audio_data);
        }

        let milli_time = start.elapsed().as_micros() as f32 / 1000.0;
        self.console.borrow_mut().output_to_console(&format!("{} took {}ms to load", name, milli_time));
    }

    pub fn get_model(&self, name: &str) -> Option<&model::Model> {

        self.models.get(name)
    }

    pub fn get_skeleton_model(&self, name: &str) -> Option<&model::SkeletonModel> {

        self.skeleton_models.get(name)
    }

    pub fn get_mut_skeleton_model(&mut self, name: &str) -> Option<&mut model::SkeletonModel> {

        self.skeleton_models.get_mut(name)
    }

    pub fn get_texture(&self, name: &str) -> Option<&texture::Texture> {

        self.textures.get(name)
    }

    pub fn get_sound(&self, name: &str) -> Option<&audio::WavAudioData> {

        self.sounds.get(name)
    }

    pub fn bulk_load(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {

        let start = std::time::Instant::now();
        
        //Make just scan folder, need better way of telling we should generate collision
        let things_to_load: Vec<(&str, bool, bool)> = vec![("./assets/cube.glb", false, false),
            ("./assets/sphere.glb", false, false),
            ("./assets/capsule.glb", false, false),
            ("./assets/cylinder.glb", false, false),
            ("./assets/test_triangle.glb", true, false),
            ("./assets/Roll_Caskett.glb", false, true),
            ("./assets/Roll_Caskett.png", false, false),
            ("./assets/dot_crosshair.png", false, false),
            ("./assets/tree.jpg", false, false),
            ("./assets/debug.png", false, false),
            ("./assets/hitsound480.wav", false, false)];

        for (path, collide, has_animation) in things_to_load {
            if path.contains(".glb") {
                if has_animation {
                    self.load_skeleton_model(device, path);
                }
                else {
                    self.load_model(device, path, collide);
                }
            }
            else if path.contains(".wav") {
                self.load_wav(path);
            }
            else {
                self.load_texture(device, queue, path);
            }
        }

        let milli_time = start.elapsed().as_micros() as f32 / 1000.0;
        self.console.borrow_mut().output_to_console(&format!("Bulk load took {}ms to load", milli_time));
    }
}