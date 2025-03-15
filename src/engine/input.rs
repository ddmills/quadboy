use std::collections::HashSet;

use bevy_ecs::prelude::*;
use macroquad::prelude::*;

#[derive(Resource, Default)]
pub struct KeyInput {
    pub down: HashSet<KeyCode>,
    pub pressed: HashSet<KeyCode>,
}

impl KeyInput {
    pub fn is_down(&self, key: KeyCode) -> bool {
        self.down.contains(&key)
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }
}

pub fn update_key_input(mut keys: ResMut<KeyInput>) {
    keys.down = get_keys_down();
    keys.pressed = get_keys_pressed();
}
