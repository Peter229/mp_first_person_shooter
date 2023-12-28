use winit::event::ScanCode;
use std::collections::HashMap;

use crate::camera;
use crate::render_commands;
use crate::collision;
use crate::input::{*, InputState};
use crate::player;
use crate::resource_manager;

enum States {
    Start,
}

const TICK_RATE: f32 = 16.66666;
const TICK_RATE_SECONDS: f32 = TICK_RATE / 1000.0;

pub struct GameState {
    current_state: States,
    delta_time: f32,
    tick_time: f32,
    current_tick: u32,
    current_time: std::time::SystemTime,
    camera: camera::Camera,
    render_commands: Vec<render_commands::RenderCommands>,
    sphere: collision::Sphere,
    capsule: collision::Capsule,
    player: player::Player,
}

impl GameState {
    
    pub fn new() -> Self {

        let current_time = std::time::SystemTime::now();

        let camera = camera::Camera::new(
            (0.0, 1.0, -4.0).into(),
            (0.0, 0.0, 0.0).into(),
            glam::f32::Vec3::Y,
            16.0 / 9.0,
            90.0_f32.to_radians(),
            0.1,
            100.0
        );

        let sphere = collision::Sphere::new(glam::f32::Vec3::new(-2.0, 0.0, 0.0), 1.0);
        let capsule = collision::Capsule::new(glam::f32::Vec3::new(0.0, -2.0, 0.0), glam::f32::Vec3::new(0.0, 2.0, 0.0), 1.0);
        let player = player::Player::new(glam::f32::Vec3::ZERO);

        Self { current_state: States::Start, delta_time: 0.0, tick_time: 0.0, current_tick: 0, current_time, camera, render_commands: Vec::new(), sphere, capsule, player }
    }

    pub fn update(&mut self, inputs: &mut Inputs, resource_manager: &resource_manager::ResourceManager) {

        self.delta_time = self.current_time.elapsed().unwrap().as_micros() as f32 / 1000.0;
        self.tick_time += self.delta_time;
        self.current_time = std::time::SystemTime::now();
        while self.tick_time >= TICK_RATE {
            self.tick_time -= TICK_RATE;
            self.begin_tick();
            self.tick(inputs, resource_manager);
            self.end_tick();
            self.current_tick += 1;
            inputs.end_tick_clean();
        }
    }

    fn begin_tick(&mut self) {

        self.render_commands.clear();
    }

    fn tick(&mut self, inputs: &mut Inputs, resource_manager: &resource_manager::ResourceManager) {

        match self.current_state {
            States::Start => {

            },
        }

        let mut input_vector = glam::f32::Vec3::ZERO;
        if inputs.check_key_down(RIGHT) {
            input_vector += glam::f32::Vec3::X;
        }
        if inputs.check_key_down(LEFT) {
            input_vector += glam::f32::Vec3::NEG_X;
        }
        if inputs.check_key_down(FORWARD) {
            input_vector += glam::f32::Vec3::Z;
        }
        if inputs.check_key_down(BACKWARD) {
            input_vector += glam::f32::Vec3::NEG_Z;
        }
        if inputs.check_key_down(UP) {
            input_vector += glam::f32::Vec3::Y;
        }
        if inputs.check_key_down(DOWN) {
            input_vector += glam::f32::Vec3::NEG_Y;
        }
        self.player.input(inputs);
        self.player.translate_relative(input_vector * TICK_RATE_SECONDS * 4.0);

        let t = self.capsule.vs_while_moving_triangle_soup(&(input_vector * TICK_RATE_SECONDS * 2.0), resource_manager.get_model(&"triangle".to_string()).unwrap().get_collision());
        self.capsule.set_center(self.player.get_position());
        if t.collided {
            println!("Collision on tick {}", self.current_tick);
        }

        self.camera.update_from_player(&self.player);
        self.render_commands.push(render_commands::RenderCommands::Camera(self.camera.build_projection_matrix().to_cols_array_2d()));

        //T * R * S
        {
            let rotation = glam::f32::Mat4::from_euler(glam::EulerRot::XYZ, self.current_tick as f32 / 10.0, self.current_tick as f32 / 10.0, 0.0);
            let translation = glam::f32::Mat4::from_translation(glam::f32::Vec3::new((self.current_tick as f32 / 20.0).sin() * 2.0, 0.0, 0.0));
            let transform = translation * rotation;
            self.render_commands.push(render_commands::RenderCommands::Model(transform, "cube".to_string(), "tree".to_string()));
        }
        {
            let rotation = glam::f32::Mat4::from_euler(glam::EulerRot::XYZ, self.current_tick as f32 / 12.0, self.current_tick as f32 / 40.0, 0.0);
            let translation = glam::f32::Mat4::from_translation(glam::f32::Vec3::new((self.current_tick as f32 / 10.0).sin() * 1.5, 2.0, 0.0));
            let transform = translation * rotation;
            self.render_commands.push(render_commands::RenderCommands::Model(transform, "cube".to_string(), "tree".to_string()));
        }

        self.render_commands.push(render_commands::RenderCommands::Model(glam::f32::Mat4::IDENTITY, "triangle".to_string(), "debug".to_string()));

        self.sphere.render(&mut self.render_commands);

        self.capsule.render(&mut self.render_commands);
    }

    fn end_tick(&mut self) {

        //If you want to save the current state put it here
    }

    pub fn get_render_commands(&self) -> &Vec<render_commands::RenderCommands> {
        
        &self.render_commands
    }
}