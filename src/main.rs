use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod render_state;
mod texture;
mod camera;
mod gpu_types;
mod game_state;
mod render_commands;
mod model;
mod resource_manager;
mod player;
mod collision;
mod input;
mod quad_renderer;

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("mp_first_person_shooter").with_inner_size(winit::dpi::PhysicalSize::new(1280, 720)).build(&event_loop).unwrap();

    window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
    window.set_cursor_visible(false);

    let mut resource_manager = resource_manager::ResourceManager::new();

    let mut render_state = pollster::block_on(render_state::RenderState::new(window));

    resource_manager.load_model(render_state.get_device(), "./assets/cube.glb", "cube", false);
    resource_manager.load_model(render_state.get_device(), "./assets/sphere.glb", "sphere", false);
    resource_manager.load_model(render_state.get_device(), "./assets/capsule.glb", "capsule", false);
    resource_manager.load_model(render_state.get_device(), "./assets/cylinder.glb", "cylinder", false);
    resource_manager.load_model(render_state.get_device(), "./assets/test_triangle.glb", "triangle", true);
    resource_manager.load_texture(render_state.get_device(), render_state.get_queue(), "./assets/dot_crosshair.png", "crosshair");
    resource_manager.load_texture(render_state.get_device(), render_state.get_queue(), "./assets/tree.jpg", "tree");
    resource_manager.load_texture(render_state.get_device(), render_state.get_queue(), "./assets/debug.png", "debug");

    let mut game_state = game_state::GameState::new();

    let mut inputs = input::Inputs::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == render_state.window().id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => { 
                render_state.resize(*physical_size); 
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => { 
                render_state.resize(**new_inner_size); 
            }
            WindowEvent::KeyboardInput { input, .. } => {
                inputs.keyboard_input(input);
            }
            _ => {}
        },
        Event::DeviceEvent { event, .. } => {
            match event {
                DeviceEvent::Motion { axis, value } => {
                    inputs.mouse_motion_input(axis, value);
                }
                DeviceEvent::Button { button, state } => {
                    inputs.mouse_input(button, state);
                }
                _ => {}
            }
        }
        Event::RedrawRequested(window_id) if window_id == render_state.window().id() => {
            match render_state.render(game_state.get_render_commands(), &resource_manager) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.get_size()),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {

            game_state.update(&mut inputs, &resource_manager);
            render_state.update_transforms(game_state.get_render_commands());
            render_state.window().request_redraw();
        }
        _ => {}
    });
}
