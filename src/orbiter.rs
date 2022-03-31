use bytemuck::{Pod, Zeroable};
use cgmath::{
    perspective, vec3, Deg, InnerSpace, Matrix4, Quaternion, SquareMatrix, Transform, Vector3, Zero,
};
use winit::event::VirtualKeyCode;

use crate::{gen::SCALE, input::Input};

pub struct Orbiter {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    rotation: Quaternion<f32>,
    perspective: Perspective,
}

impl Default for Orbiter {
    fn default() -> Self {
        Self {
            position: vec3(0.0, 0.0, SCALE * 2.0),
            velocity: Vector3::zero(),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            perspective: Perspective::default(),
        }
    }
}

impl Orbiter {
    pub fn process(&mut self, input: &Input) {
        {
            let gravity_dir = -self.position.normalize();
            let gravity_strength = 10.0 / self.position.magnitude2();
            self.velocity += gravity_dir * gravity_strength;
        }

        const SPEED: f32 = 0.01;

        if input.is_key_down(VirtualKeyCode::W) {
            self.velocity += self.forward() * SPEED;
        }

        if input.is_key_down(VirtualKeyCode::S) {
            self.velocity -= self.forward() * SPEED;
        }

        if input.is_key_down(VirtualKeyCode::A) {
            self.velocity -= self.right() * SPEED;
        }

        if input.is_key_down(VirtualKeyCode::D) {
            self.velocity += self.right() * SPEED;
        }

        if input.is_key_down(VirtualKeyCode::Q) {
            self.velocity -= self.up() * SPEED;
        }

        if input.is_key_down(VirtualKeyCode::E) {
            self.velocity += self.up() * SPEED;
        }

        self.position += self.velocity;
    }

    pub fn recalc(&mut self, width: u32, height: u32) {
        self.perspective.recalc(width, height);
    }

    pub fn matrices(&self) -> Matrices {
        let rotation: Matrix4<f32> = self.rotation.into();
        let view_to_world = Matrix4::from_translation(self.position) * rotation;
        let world_to_clip = self.perspective.matrix * view_to_world.inverse_transform().unwrap();

        Matrices {
            world_to_clip,
            view_to_world,
        }
    }

    pub fn position(&self) -> Vector3<f32> {
        self.position
    }

    fn forward(&self) -> Vector3<f32> {
        self.rotation * -Vector3::unit_z()
    }

    fn right(&self) -> Vector3<f32> {
        self.rotation * Vector3::unit_x()
    }

    fn up(&self) -> Vector3<f32> {
        self.rotation * Vector3::unit_y()
    }
}

struct Perspective {
    matrix: Matrix4<f32>,
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            matrix: Matrix4::identity(),
        }
    }
}

impl Perspective {
    fn recalc(&mut self, width: u32, height: u32) {
        let (width, height) = (width as f32, height as f32);
        self.matrix = perspective(Deg(80.0), width / height, 0.01, 512.0);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Matrices {
    world_to_clip: Matrix4<f32>,
    view_to_world: Matrix4<f32>,
}

unsafe impl Zeroable for Matrices {}
unsafe impl Pod for Matrices {}
