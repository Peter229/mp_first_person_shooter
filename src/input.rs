use winit::{self, keyboard::{KeyCode, PhysicalKey}, event::{ElementState, KeyEvent}};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum InputState {
    JustPressed,
    Held,
    JustReleased,
    Released,
}

pub const FORWARD: KeyCode = KeyCode::KeyW;
pub const LEFT: KeyCode = KeyCode::KeyA;
pub const BACKWARD: KeyCode = KeyCode::KeyS;
pub const RIGHT: KeyCode = KeyCode::KeyD;
pub const DOWN: KeyCode = KeyCode::KeyQ;
pub const UP: KeyCode = KeyCode::KeyE;

pub const MOUSE_SENSITIVITY: f32 = 0.003;

pub struct Inputs {
    keyboard_inputs: HashMap<KeyCode, InputState>,
    mouse_buttons: HashMap<u32, InputState>,
    mouse_motion: [f32; 2],
}

impl Inputs {

    pub fn new() -> Self {

        let keyboard_inputs: HashMap<KeyCode, InputState> = HashMap::new();
        let mouse_buttons: HashMap<u32, InputState> = HashMap::new();
        let mouse_motion: [f32; 2] = [0.0, 0.0];

        Self { keyboard_inputs, mouse_buttons, mouse_motion }
    }

    pub fn keyboard_input(&mut self, input: &KeyEvent) {

        let key = match input.physical_key {
            PhysicalKey::Code(key_code) => {
                Some(key_code)
            }
            _ => None,
        };

        if key.is_none() {
            println!("Key {:?} doesn't exist", input.physical_key);
            return;
        }

        let input_key = key.unwrap();

        if input.state == ElementState::Pressed {
            if self.keyboard_inputs.get(&input_key).is_some() {
                let state = self.keyboard_inputs.get_mut(&input_key).unwrap();
                if *state == InputState::JustPressed {
                    *state = InputState::Held;
                }
                else if *state != InputState::Held {
                    *state = InputState::JustPressed;
                }
            }
            else {
                self.keyboard_inputs.insert(input_key, InputState::JustPressed);
            }
        }
        else {

            self.keyboard_inputs.insert(input_key, InputState::JustReleased);
        }
    }

    pub fn mouse_input(&mut self, button: u32, state: ElementState) {

        if state == ElementState::Pressed {
            if self.mouse_buttons.get(&button).is_some() {
                let state = self.mouse_buttons.get_mut(&button).unwrap();
                if *state == InputState::JustPressed {
                    *state = InputState::Held;
                }
                else if *state != InputState::Held {
                    *state = InputState::JustPressed;
                }
            }
            else {
                self.mouse_buttons.insert(button, InputState::JustPressed);
            }
        }
        else {

            self.mouse_buttons.insert(button, InputState::JustReleased);
        }
    }

    pub fn mouse_motion_input(&mut self, axis: u32, value: f64) {

        self.mouse_motion[axis as usize] += value as f32;
    }

    pub fn end_tick_clean(&mut self) {

        for (_, input_state) in self.keyboard_inputs.iter_mut() {

            if *input_state == InputState::JustPressed {
                *input_state = InputState::Held;
            }
            if *input_state == InputState::JustReleased {
                *input_state = InputState::Released;
            }
        }

        for (_, input_state) in self.mouse_buttons.iter_mut() {

            if *input_state == InputState::JustPressed {
                *input_state = InputState::Held;
            }
            if *input_state == InputState::JustReleased {
                *input_state = InputState::Released;
            }
        }

        self.mouse_motion = [0.0, 0.0];
    }

    pub fn get_mouse_motion(&self) -> [f32; 2] {

        self.mouse_motion
    }

    pub fn check_key_down(&self, key: KeyCode) -> bool {

        self.keyboard_inputs.get(&key).is_some_and(|input| *input == InputState::JustPressed || *input == InputState::Held)
    }

    pub fn check_mouse_down(&self, key: u32) -> bool {

        self.mouse_buttons.get(&key).is_some_and(|input| *input == InputState::JustPressed || *input == InputState::Held)
    }

    pub fn check_mouse_just_pressed(&self, key: u32) -> bool {

        self.mouse_buttons.get(&key).is_some_and(|input| *input == InputState::JustPressed)
    }
}