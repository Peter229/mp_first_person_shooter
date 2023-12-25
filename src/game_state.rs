use crate::camera;
use crate::render_commands;

enum States {
    Start,
}

const TICK_RATE: f32 = 16.66666;

pub struct GameState {
    current_state: States,
    delta_time: f32,
    tick_time: f32,
    current_tick: u32,
    current_time: std::time::SystemTime,
    camera: camera::Camera,
    render_commands: Vec<render_commands::RenderCommands>,
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

        Self { current_state: States::Start, delta_time: 0.0, tick_time: 0.0, current_tick: 0, current_time, camera, render_commands: Vec::new() }
    }

    pub fn update(&mut self) {

        self.delta_time = self.current_time.elapsed().unwrap().as_micros() as f32 / 1000.0;
        self.tick_time += self.delta_time;
        self.current_time = std::time::SystemTime::now();
        while self.tick_time >= TICK_RATE {
            self.tick_time -= TICK_RATE;
            self.begin_tick();
            self.tick();
            self.end_tick();
            self.current_tick += 1;
        }
    }

    fn begin_tick(&mut self) {

        self.render_commands.clear();
    }

    fn tick(&mut self) {

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
    }

    fn end_tick(&mut self) {
         
        //If you want to save the current state put it here
    }

    pub fn get_render_commands(&self) -> &Vec<render_commands::RenderCommands> {
        
        &self.render_commands
    }
}