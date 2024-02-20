use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::camera;
use crate::console::Console;
use crate::render_commands::*;
use crate::collision;
use crate::input::*;
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
    render_commands: Vec<RenderCommands>,
    sphere: collision::Sphere,
    capsule: collision::Capsule,
    player: player::Player,
    hit_areas: Vec<collision::Sphere>,
    console: Rc<RefCell<Console>>,
}

impl GameState {
    
    pub fn new(console: Rc<RefCell<Console>>) -> Self {

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
        let capsule = collision::Capsule::new(glam::f32::Vec3::new(0.0, 1.0, 0.0), glam::f32::Vec3::new(0.0, 5.0, 0.0), 1.0);
        let player = player::Player::new(glam::f32::Vec3::new(0.0, 1.0, 4.0), -90.0_f32.to_radians());

        Self { current_state: States::Start, delta_time: 0.0, tick_time: 0.0, current_tick: 0, current_time, camera, render_commands: Vec::new(), sphere, capsule, player, hit_areas: Vec::new(), console }
    }

    pub fn update(&mut self, inputs: &mut Inputs, resource_manager: &mut resource_manager::ResourceManager) {

        self.delta_time = self.current_time.elapsed().unwrap().as_micros() as f32 / 1000.0;
        self.tick_time += self.delta_time;
        self.current_time = std::time::SystemTime::now();
        while self.tick_time >= TICK_RATE {
            let start = Instant::now();
            self.tick_time -= TICK_RATE;
            self.begin_tick();
            self.tick(inputs, resource_manager);
            self.end_tick();
            self.current_tick += 1;
            inputs.end_tick_clean();
            let milli_time = start.elapsed().as_micros() as f32 / 1000.0;
            self.console.borrow_mut().insert_timing("Game tick", milli_time);
        }
    }

    fn begin_tick(&mut self) {

        self.render_commands.clear();
    }

    fn tick(&mut self, inputs: &mut Inputs, resource_manager: &mut resource_manager::ResourceManager) {

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

        let t = self.capsule.vs_while_moving_triangle_soup(&(glam::f32::Vec3::NEG_Y * 2.0), resource_manager.get_model(&"test_triangle".to_string()).unwrap().get_collision());
        if t.collided {
            self.capsule.set_center(self.capsule.get_center() + (glam::f32::Vec3::NEG_Y * 2.0) * t.penetration_or_time + t.normal * std::f32::EPSILON);
            //println!("Collision on tick {}", self.current_tick);
        }

        //Render area of game
        self.camera.update_from_player(&self.player);
        self.render_commands.push(RenderCommands::Camera(self.camera.build_projection_matrix().to_cols_array_2d()));

        //T * R * S
        {
            let rotation = glam::f32::Mat4::from_euler(glam::EulerRot::XYZ, self.current_tick as f32 / 10.0, self.current_tick as f32 / 10.0, 0.0);
            let translation = glam::f32::Mat4::from_translation(glam::f32::Vec3::new((self.current_tick as f32 / 20.0).sin() * 2.0, 7.0, 0.0));
            let transform = translation * rotation;
            self.render_commands.push(RenderCommands::Model(ModelRenderCommand::new(transform, "cube", "tree")));
        }
        {
            let rotation = glam::f32::Mat4::from_euler(glam::EulerRot::XYZ, self.current_tick as f32 / 12.0, self.current_tick as f32 / 40.0, 0.0);
            let translation = glam::f32::Mat4::from_translation(glam::f32::Vec3::new((self.current_tick as f32 / 10.0).sin() * 1.5, 10.0, 0.0));
            let transform = translation * rotation;
            self.render_commands.push(RenderCommands::Model(ModelRenderCommand::new(transform, "cube", "tree")));
        }

        self.render_commands.push(RenderCommands::Model(ModelRenderCommand::new(glam::f32::Mat4::IDENTITY, "test_triangle", "debug")));

        self.sphere.render(&mut self.render_commands);

        self.capsule.render(&mut self.render_commands);

        for sphere in &self.hit_areas {
            sphere.render(&mut self.render_commands);
        }

        let roll_model_matrix = glam::f32::Mat4::from_scale_rotation_translation(glam::f32::Vec3::new(0.1, 0.1, 0.1), glam::f32::Quat::from_rotation_x(90.0_f32.to_radians()), glam::f32::Vec3::new(2.0, 0.0, 0.0));

        resource_manager.get_mut_skeleton_model(&"Roll_Caskett".to_string()).unwrap().update_skeleton(TICK_RATE_SECONDS);
        self.render_commands.push(RenderCommands::SkeletonModel(SkeletonModelRenderCommand::new(roll_model_matrix, "Roll_Caskett", "Roll_Caskett")));

        self.render_commands.push(RenderCommands::Quad(glam::f32::Vec3::new(-0.005, -0.005, 0.0), glam::f32::Vec3::new(0.005, 0.005, 0.0), "dot_crosshair".to_string()));
    }

    fn end_tick(&mut self) {

        //If you want to save the current state put it here
    }

    pub fn get_render_commands(&self) -> &Vec<RenderCommands> {
        
        &self.render_commands
    }

    pub fn get_mut_render_commands(&mut self) -> &mut Vec<RenderCommands> {
        
        &mut self.render_commands
    }

    pub fn get_delta_time(&self) -> f32 {
        self.delta_time
    }
}