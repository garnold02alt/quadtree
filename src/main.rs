mod gen;
mod input;
mod orbiter;
mod render;
mod tree;

use cgmath::vec2;
use input::Input;
use orbiter::Orbiter;
use tree::Tree;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, KeyboardInput, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::default().build(&event_loop).unwrap();

    let mut renderer = render::init(&window);
    let mut input = Input::default();
    let mut orbiter = Orbiter::default();
    let mut tree = Tree::new(&renderer);

    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *flow = ControlFlow::Exit;
                }

                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    renderer.configure(width, height);
                    orbiter.recalc(width, height);
                }

                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => input.key(key, state),

                WindowEvent::CursorMoved {
                    position: PhysicalPosition { x, y },
                    ..
                } => input.movement(vec2(x as f32, y as f32)),

                WindowEvent::MouseWheel { delta, .. } => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(_, delta) => delta,
                        MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => y as f32,
                    };
                    input.scroll(delta);
                }

                WindowEvent::MouseInput { button, state, .. } => input.button(button, state),

                _ => (),
            },

            Event::MainEventsCleared => {
                orbiter.process(&input);
                tree.process(&orbiter, &renderer);

                let mut meshes = Vec::new();
                tree.collect_meshes(&mut meshes);

                renderer.render(&orbiter, &meshes);
                input.process();
            }

            _ => (),
        }
    });
}
