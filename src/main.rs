use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod render_state;
mod texture;

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut render_state = pollster::block_on(render_state::RenderState::new(window));

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
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == render_state.window().id() => {
            match render_state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.get_size()),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            render_state.window().request_redraw();
        }
        _ => {}
    });
}
