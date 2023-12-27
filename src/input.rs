use winit;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum InputState {
    JustPressed,
    Held,
    JustReleased,
    Released,
}

pub fn check_key_down(inputs: &mut HashMap<winit::event::ScanCode, InputState>, key: winit::event::ScanCode) -> bool {
    inputs.get(&key).is_some_and(|input| *input == InputState::JustPressed || *input == InputState::Held)
}

pub const FORWARD: winit::event::ScanCode = 17;
pub const LEFT: winit::event::ScanCode = 30;
pub const BACKWARD: winit::event::ScanCode = 31;
pub const RIGHT: winit::event::ScanCode = 32;