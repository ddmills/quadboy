use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::*;
use macroquad::prelude::*;

#[derive(Resource, Default)]
pub struct KeyInput {
    pub down: HashSet<KeyCode>,
    pub pressed: HashSet<KeyCode>,
    pub released: HashSet<KeyCode>,
}

#[allow(dead_code)]
impl KeyInput {
    pub fn is_down(&self, key: KeyCode) -> bool {
        self.down.contains(&key)
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn is_released(&self, key: KeyCode) -> bool {
        self.released.contains(&key)
    }

    pub fn any_down(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|key| self.down.contains(key))
    }

    pub fn any_pressed(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|key| self.pressed.contains(key))
    }

    pub fn any_released(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|key| self.released.contains(key))
    }
}

#[derive(Default)]
pub struct KeyState {
    pub time: f64,
    pub delayed: bool,
}

#[derive(Default)]
pub struct InputRate {
    pub keys: HashMap<KeyCode, KeyState>, // TODO make keys private
}

impl InputRate {
    pub fn try_key(&mut self, key: KeyCode, now: f64, rate: f64, delay: f64) -> bool {
        if let Some(s) = self.keys.get(&key) {
            let t = match s.delayed {
                true => rate,
                false => delay,
            };

            if now - s.time > t {
                self.keys.insert(
                    key,
                    KeyState {
                        time: now,
                        delayed: true,
                    },
                );

                return true;
            }

            return false;
        };

        self.keys.insert(
            key,
            KeyState {
                time: now,
                delayed: false,
            },
        );
        true
    }
}

pub fn update_key_input(mut keys: ResMut<KeyInput>) {
    keys.down = get_keys_down();
    keys.pressed = get_keys_pressed();
    keys.released = get_keys_released();
}
