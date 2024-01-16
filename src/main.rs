use egui::ClippedPrimitive;
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
mod collision_world;

//Look at cpal for audio

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().with_title("mp_first_person_shooter").with_inner_size(winit::dpi::PhysicalSize::new(1280, 720)).build(&event_loop).unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
    let mut cursor_visible = false;
    window.set_cursor_visible(cursor_visible);

    let mut render_state = pollster::block_on(render_state::RenderState::new(window));

    let mut resource_manager = resource_manager::ResourceManager::new();

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



    //EGUI

    let mut egui_context = egui::Context::default();

    let mut platform = egui_winit::State::new(egui_context.clone(), egui::ViewportId::ROOT, &event_loop, Some(render_state.get_window().scale_factor() as f32), None);

    let mut renderer = egui_wgpu::Renderer::new(render_state.get_device(), render_state.get_config().format, None, 1);
    //EGUI

    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == render_state.get_window().id() => {

            if cursor_visible {

                let _ = platform.on_window_event(render_state.get_window(), event);
            }

            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                            ..
                        },
                    ..
                } => elwt.exit(),
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Tab),
                            ..
                        },
                    ..
                } => {
                    //window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
                    cursor_visible = !cursor_visible;
                    render_state.get_window().set_cursor_visible(cursor_visible);
                },
                WindowEvent::Resized(physical_size) => { 
                    render_state.resize(*physical_size); 
                }
                //This is mad at me for no reason
                //WindowEvent::ScaleFactorChanged { mut inner_size_writer, .. } => { 
                //    inner_size_writer.request_inner_size(render_state.get_size()).unwrap();
                //}
                WindowEvent::KeyboardInput { event, .. } => {
                    inputs.keyboard_input(event);
                }
                WindowEvent::RedrawRequested => {

                    //EGUI
                    let raw_input = platform.take_egui_input(render_state.get_window());
                    let full_output = egui_context.run(raw_input, |egui_context| {
                        egui::Window::new("My Window").show(&egui_context, |ui| {
                            ui.label("Hello world");
                            if ui.button("Click me").clicked() {
                                println!("hi");
                            }
                        });
                    });
                    platform.handle_platform_output(render_state.get_window(), full_output.platform_output);
                    let clipped_primitives = egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);

                    //END MY PAIN
                    match render_state.render(game_state.get_render_commands(), &resource_manager, &mut renderer, &egui_context, &clipped_primitives, full_output.pixels_per_point, &full_output.textures_delta) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.get_size()),
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            }
        }
        Event::DeviceEvent { event, .. } => {

            if !cursor_visible {
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
        }
        Event::AboutToWait => {
            game_state.update(&mut inputs, &resource_manager);
            render_state.update_transforms(game_state.get_render_commands());
            render_state.get_window().request_redraw();
        }
        _ => {}
    }).unwrap();
}
