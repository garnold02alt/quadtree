mod render;

use std::rc::Rc;

use cgmath::vec3;
use render::Vertex;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::default().build(&event_loop).unwrap();
    let render_state = render::init(&window);

    let mesh = Rc::new(render_state.create_mesh(
        &[
            Vertex {
                position: vec3(0.0, 0.0, 0.0),
                normal: vec3(0.0, 0.0, 0.0),
            },
            Vertex {
                position: vec3(1.0, 0.0, 0.0),
                normal: vec3(0.0, 0.0, 0.0),
            },
            Vertex {
                position: vec3(0.0, 1.0, 0.0),
                normal: vec3(0.0, 0.0, 0.0),
            },
        ],
        &[[0, 1, 2]],
    ));

    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    render_state.configure(width, height);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => {}
                WindowEvent::CursorMoved {
                    position: PhysicalPosition { x, y },
                    ..
                } => {}
                _ => (),
            },
            Event::MainEventsCleared => {
                render_state.render(&[mesh.clone()]);
            }
            _ => (),
        }
    });
}
