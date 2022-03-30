use bytemuck::{Pod, Zeroable};
use cgmath::{
    perspective, vec2, vec3, Deg, Matrix3, Matrix4, SquareMatrix, Transform, Vector2, Vector3,
};
use winit::event::{MouseButton, VirtualKeyCode};

use crate::input::Input;

pub struct Camera {
    position: Vector3<f32>,
    rotation: Vector2<f32>,
    projection: Matrix4<f32>,
    speed: i32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: vec3(0.0, 0.0, 5.0),
            rotation: vec2(0.0, 0.0),
            projection: Matrix4::identity(),
            speed: 50,
        }
    }
}

impl Camera {
    pub fn recalc(&mut self, width: u32, height: u32) {
        let (width, height) = (width as f32, height as f32);
        self.projection = perspective(Deg(80.0), width / height, 0.01, 512.0);
    }

    pub fn matrices(&self) -> Matrices {
        let view_to_world = Matrix4::from_translation(self.position)
            * Matrix4::from_angle_y(Deg(self.rotation.y))
            * Matrix4::from_angle_x(Deg(self.rotation.x));

        let world_to_clip = self.projection * view_to_world.inverse_transform().unwrap();

        Matrices {
            world_to_clip,
            view_to_world,
        }
    }

    pub fn move_forward(&mut self, delta: f32) {
        self.position += self.forward() * self.speed_multiplier() * delta;
    }

    pub fn move_backward(&mut self, delta: f32) {
        self.position -= self.forward() * self.speed_multiplier() * delta;
    }

    pub fn move_right(&mut self, delta: f32) {
        self.position += self.right() * self.speed_multiplier() * delta;
    }

    pub fn move_left(&mut self, delta: f32) {
        self.position -= self.right() * self.speed_multiplier() * delta;
    }

    pub fn move_up(&mut self, delta: f32) {
        self.position += Vector3::unit_y() * self.speed_multiplier() * delta;
    }

    pub fn move_down(&mut self, delta: f32) {
        self.position -= Vector3::unit_y() * self.speed_multiplier() * delta;
    }

    pub fn look(&mut self, mouse_delta: Vector2<f32>, delta: f32) {
        const SENSITIVITY: f32 = 10.0;
        self.rotation.y -= mouse_delta.x * SENSITIVITY * delta;
        self.rotation.x =
            (self.rotation.x - mouse_delta.y * SENSITIVITY * delta).clamp(-90.0, 90.0);
    }

    pub fn increase_speed(&mut self) {
        self.speed += 1;
    }

    pub fn decrease_speed(&mut self) {
        self.speed -= 1;
    }

    pub fn forward(&self) -> Vector3<f32> {
        Matrix3::from_angle_y(Deg(self.rotation.y))
            * Matrix3::from_angle_x(Deg(self.rotation.x))
            * -Vector3::unit_z()
    }

    fn right(&self) -> Vector3<f32> {
        Matrix3::from_angle_y(Deg(self.rotation.y))
            * Matrix3::from_angle_x(Deg(self.rotation.x))
            * Vector3::unit_x()
    }

    fn speed_multiplier(&self) -> f32 {
        8.0 * 1.1f32.powi(self.speed - 50)
    }
}

pub fn control(input: &Input, camera: &mut Camera) {
    if !input.is_button_down(MouseButton::Right) {
        return;
    }

    let delta = 1.0 / 60.0;

    if input.is_key_down(VirtualKeyCode::W) {
        camera.move_forward(delta);
    }

    if input.is_key_down(VirtualKeyCode::S) {
        camera.move_backward(delta);
    }

    if input.is_key_down(VirtualKeyCode::A) {
        camera.move_left(delta);
    }

    if input.is_key_down(VirtualKeyCode::D) {
        camera.move_right(delta);
    }

    if input.is_key_down(VirtualKeyCode::Q) {
        camera.move_down(delta);
    }

    if input.is_key_down(VirtualKeyCode::E) {
        camera.move_up(delta);
    }

    if input.mouse_wheel().abs() > 0.1 {
        if input.mouse_wheel() > 0.0 {
            camera.increase_speed();
        } else {
            camera.decrease_speed();
        }
    }

    camera.look(input.mouse_delta(), delta);
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Matrices {
    world_to_clip: Matrix4<f32>,
    view_to_world: Matrix4<f32>,
}

unsafe impl Zeroable for Matrices {}
unsafe impl Pod for Matrices {}
