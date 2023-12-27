use winit::event::ScanCode;
use std::collections::HashMap;

use crate::camera;
use crate::render_commands;
use crate::collision;
use crate::input::{*, InputState};
use crate::player;

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
            (0.0, 0.0, -4.0).into(),
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

    pub fn update(&mut self, inputs: &mut HashMap<ScanCode, InputState>) {

        self.delta_time = self.current_time.elapsed().unwrap().as_micros() as f32 / 1000.0;
        self.tick_time += self.delta_time;
        self.current_time = std::time::SystemTime::now();
        while self.tick_time >= TICK_RATE {
            self.tick_time -= TICK_RATE;
            self.begin_tick();
            self.tick(inputs);
            self.end_tick();
            self.current_tick += 1;
            for (_, input_state) in inputs.iter_mut() {

                if *input_state == InputState::JustPressed {
                    *input_state = InputState::Held;
                }
                if *input_state == InputState::JustReleased {
                    *input_state = InputState::Released;
                }
            }
        }
    }

    fn begin_tick(&mut self) {

        self.render_commands.clear();
    }

    fn tick(&mut self, inputs: &mut HashMap<ScanCode, InputState>) {

        match self.current_state {
            States::Start => {

            },
        }

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

        self.sphere.render(&mut self.render_commands);

        let mut input_vector = glam::f32::Vec3::ZERO;
        if check_key_down(inputs, RIGHT) {
            input_vector += glam::f32::Vec3::X;
        }
        if check_key_down(inputs, LEFT) {
            input_vector += glam::f32::Vec3::NEG_X;
        }
        if check_key_down(inputs, FORWARD) {
            input_vector += glam::f32::Vec3::Z;
        }
        if check_key_down(inputs, BACKWARD) {
            input_vector += glam::f32::Vec3::NEG_Z;
        }
        self.player.translate(input_vector * TICK_RATE_SECONDS * 2.0);
        self.capsule.set_center(self.player.get_position());
        self.capsule.render(&mut self.render_commands);

        let t = self.sphere.vs_capsule(&self.capsule);
        if t.collided {
            println!("Collision");
        }
        else {
            println!("No collision");
        }
    }

    fn end_tick(&mut self) {

        //If you want to save the current state put it here
    }

    pub fn get_render_commands(&self) -> &Vec<render_commands::RenderCommands> {
        
        &self.render_commands
    }
}