use std::{cell::RefCell, rc::Rc};

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
mod console;
mod audio;

//Look at cpal for audio

fn main() {
    env_logger::init();
    
    //I wish this and the resource manager could be global without unsafe 
    let console = Rc::new(RefCell::new(console::Console::new()));

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().with_title("mp_first_person_shooter").with_inner_size(winit::dpi::PhysicalSize::new(1280, 720)).build(&event_loop).unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
    let mut cursor_visible = false;
    window.set_cursor_visible(cursor_visible);

    let mut render_state = pollster::block_on(render_state::RenderState::new(window, console.clone()));

    let mut resource_manager = resource_manager::ResourceManager::new(console.clone());

    resource_manager.bulk_load(render_state.get_device(), render_state.get_queue());

    let mut game_state = game_state::GameState::new(console.clone());

    let mut inputs = input::Inputs::new();

    let mut audio = audio::AudioState::new();


    //EGUI
    let egui_context = egui::Context::default();

    let mut platform = egui_winit::State::new(egui_context.clone(), egui::ViewportId::ROOT, &event_loop, Some(render_state.get_window().scale_factor() as f32), None);

    let mut renderer = egui_wgpu::Renderer::new(render_state.get_device(), render_state.get_config().format, None, 1);
    //EGUI

    let mut console_text = "".to_string();

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
                    audio.play_wav();
                    cursor_visible = !cursor_visible;
                    render_state.get_window().set_cursor_visible(cursor_visible);
                    if cursor_visible {
                        render_state.get_window().set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                        render_state.get_window().set_cursor_position(winit::dpi::LogicalPosition::new(render_state.get_config().width / 2, render_state.get_config().height / 2)).unwrap();
                    }
                    else
                    {
                        render_state.get_window().set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
                    }
                },
                WindowEvent::Resized(physical_size) => { 
                    render_state.resize(*physical_size); 
                }
                //This is mad at me for no reason
                //WindowEvent::ScaleFactorChanged { mut inner_size_writer, .. } => { 
                //    inner_size_writer.request_inner_size(render_state.get_size()).unwrap();
                //}
                WindowEvent::KeyboardInput { event, .. } => {
                    if !cursor_visible {
                        inputs.keyboard_input(event);
                    }
                }
                WindowEvent::RedrawRequested => {

                    //EGUI
                    let mut selected = resource_manager.get_skeleton_model("Roll_Caskett").unwrap().get_animation_controller().get_current_animation().to_owned();
                    let raw_input = platform.take_egui_input(render_state.get_window());
                    let full_output = egui_context.run(raw_input, |egui_context| {
                        egui::Window::new("Debug Stats").show(&egui_context, |ui| {
                            ui.label("Frame time: ".to_string() + &game_state.get_delta_time().to_string());
                            ui.label("FPS: ".to_string() + &(1.0 / (game_state.get_delta_time() / 1000.0)).to_string());
                            ui.label("Number of render commands: ".to_string() + &(game_state.get_render_commands().len().to_string()));
                            ui.label(console.borrow().get_timings_string());
                            egui::ComboBox::from_label("Current animation").selected_text(format!("{:?}", selected)).show_ui(ui, |ui| {
                                for anim in resource_manager.get_skeleton_model("Roll_Caskett").unwrap().get_animation_controller().get_animations() {
                                    ui.selectable_value(&mut selected, anim.to_string(), anim);
                                }
                            })
                        });
                        egui::Window::new("Console").fixed_size(egui::Vec2::new(300.0, 400.0)).show(&egui_context, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.label(console.borrow().get_log());
                            });
                            if ui.text_edit_singleline(&mut console_text).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                if !console_text.is_empty() {
                                    console.borrow_mut().output_to_console(&console_text);
                                    console_text.clear();

                                }
                            }
                        });
                    });
                    resource_manager.get_mut_skeleton_model("Roll_Caskett").unwrap().get_mut_animation_controller().set_current_animation(&selected);
                    platform.handle_platform_output(render_state.get_window(), full_output.platform_output);
                    let clipped_primitives = egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);

                    match render_state.render(game_state.get_render_commands(), &resource_manager, &mut renderer, &clipped_primitives, full_output.pixels_per_point, &full_output.textures_delta) {
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
            game_state.update(&mut inputs, &mut resource_manager);
            render_state.update_transforms(game_state.get_mut_render_commands());
            render_state.get_window().request_redraw();
        }
        _ => {}
    }).unwrap();
}
