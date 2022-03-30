use std::collections::HashMap;

use cgmath::{Vector2, Zero};
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

pub struct Input {
    key_states: HashMap<VirtualKeyCode, ActionState>,
    button_states: HashMap<MouseButton, ActionState>,
    mouse_pos_before: Vector2<f32>,
    mouse_pos: Vector2<f32>,
    mouse_wheel: f32,
    delta_override: Option<Vector2<f32>>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            key_states: HashMap::new(),
            button_states: HashMap::new(),
            mouse_pos_before: Vector2::zero(),
            mouse_pos: Vector2::zero(),
            mouse_wheel: 0.0,
            delta_override: None,
        }
    }
}

#[allow(dead_code)]
impl Input {
    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        self.key_states
            .get(&key)
            .map(|state| state.is_down())
            .unwrap_or(false)
    }

    pub fn is_key_down_once(&self, key: VirtualKeyCode) -> bool {
        self.key_states
            .get(&key)
            .map(|state| state.is_down_once())
            .unwrap_or(false)
    }

    pub fn was_key_down_once(&self, key: VirtualKeyCode) -> bool {
        self.key_states
            .get(&key)
            .map(|state| state.was_down_once())
            .unwrap_or(false)
    }

    pub fn is_button_down(&self, button: MouseButton) -> bool {
        self.button_states
            .get(&button)
            .map(|state| state.is_down())
            .unwrap_or(false)
    }

    pub fn is_button_down_once(&self, button: MouseButton) -> bool {
        self.button_states
            .get(&button)
            .map(|state| state.is_down_once())
            .unwrap_or(false)
    }

    pub fn was_button_down_once(&self, button: MouseButton) -> bool {
        self.button_states
            .get(&button)
            .map(|state| state.was_down_once())
            .unwrap_or(false)
    }

    pub fn mouse_pos(&self) -> Vector2<f32> {
        self.mouse_pos
    }

    pub fn mouse_delta(&self) -> Vector2<f32> {
        self.delta_override
            .unwrap_or_else(|| self.mouse_pos - self.mouse_pos_before)
    }

    pub fn mouse_wheel(&self) -> f32 {
        self.mouse_wheel
    }

    pub fn process(&mut self) {
        for state in self.key_states.values_mut() {
            state.increment();
        }

        for state in self.button_states.values_mut() {
            state.increment();
        }

        self.mouse_pos_before = self.mouse_pos;
        self.mouse_wheel = 0.0;
        self.delta_override = None;
    }

    pub fn key(&mut self, key: VirtualKeyCode, state: ElementState) {
        self.key_states.entry(key).or_default().set(state);
    }

    pub fn button(&mut self, button: MouseButton, state: ElementState) {
        self.button_states.entry(button).or_default().set(state);
    }

    pub fn movement(&mut self, new_pos: Vector2<f32>) {
        self.mouse_pos = new_pos;
    }

    pub fn movement_override(&mut self, delta: Vector2<f32>) {
        self.delta_override = Some(delta);
    }

    pub fn scroll(&mut self, movement: f32) {
        self.mouse_wheel = movement;
    }
}

enum ActionState {
    Active(i32),
    Inactive(i32),
}

impl Default for ActionState {
    fn default() -> Self {
        Self::Inactive(0)
    }
}

impl ActionState {
    fn set(&mut self, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if matches!(self, Self::Inactive(_)) {
                    *self = Self::Active(0);
                }
            }
            ElementState::Released => {
                if matches!(self, Self::Active(_)) {
                    *self = Self::Inactive(0);
                }
            }
        }
    }

    fn increment(&mut self) {
        match self {
            ActionState::Active(t) => *t += 1,
            ActionState::Inactive(t) => *t += 1,
        }
    }

    fn is_down(&self) -> bool {
        matches!(self, ActionState::Active(_))
    }

    fn is_down_once(&self) -> bool {
        if let ActionState::Active(t) = self {
            *t == 0
        } else {
            false
        }
    }

    fn was_down_once(&self) -> bool {
        if let ActionState::Inactive(t) = self {
            *t == 0
        } else {
            false
        }
    }
}
