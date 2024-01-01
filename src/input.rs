use winit::{self, event::{ScanCode, KeyboardInput, ElementState, DeviceEvent}};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum InputState {
    JustPressed,
    Held,
    JustReleased,
    Released,
}

pub const FORWARD: winit::event::ScanCode = 17;
pub const LEFT: winit::event::ScanCode = 30;
pub const BACKWARD: winit::event::ScanCode = 31;
pub const RIGHT: winit::event::ScanCode = 32;
pub const DOWN: winit::event::ScanCode = 16;
pub const UP: winit::event::ScanCode = 18;

pub const MOUSE_SENSITIVITY: f32 = 0.003;

pub struct Inputs {
    keyboard_inputs: HashMap<ScanCode, InputState>,
    mouse_buttons: HashMap<u32, InputState>,
    mouse_motion: [f32; 2],
}

impl Inputs {

    pub fn new() -> Self {

        let keyboard_inputs: HashMap<ScanCode, InputState> = HashMap::new();
        let mouse_buttons: HashMap<u32, InputState> = HashMap::new();
        let mouse_motion: [f32; 2] = [0.0, 0.0];

        Self { keyboard_inputs, mouse_buttons, mouse_motion }
    }

    pub fn keyboard_input(&mut self, input: &KeyboardInput) {

        if input.state == ElementState::Pressed {
            if self.keyboard_inputs.get(&input.scancode).is_some() {
                let state = self.keyboard_inputs.get_mut(&input.scancode).unwrap();
                if *state == InputState::JustPressed {
                    *state = InputState::Held;
                }
                else if *state != InputState::Held {
                    *state = InputState::JustPressed;
                }
            }
            else {
                self.keyboard_inputs.insert(input.scancode, InputState::JustPressed);
            }
        }
        else {

            self.keyboard_inputs.insert(input.scancode, InputState::JustReleased);
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

    pub fn check_key_down(&self, key: ScanCode) -> bool {

        self.keyboard_inputs.get(&key).is_some_and(|input| *input == InputState::JustPressed || *input == InputState::Held)
    }

    pub fn check_mouse_down(&self, key: ScanCode) -> bool {

        self.mouse_buttons.get(&key).is_some_and(|input| *input == InputState::JustPressed || *input == InputState::Held)
    }

    pub fn check_mouse_just_pressed(&self, key: ScanCode) -> bool {

        self.mouse_buttons.get(&key).is_some_and(|input| *input == InputState::JustPressed)
    }
}