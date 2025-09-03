use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{engine::KeyInput, ui::Interaction};

#[derive(Component, Clone, Debug)]
pub struct Callback(pub SystemId);

#[derive(Component, Clone, Debug)]
pub struct Triggered;

#[derive(Component, Clone, Debug)]
pub struct Hotkey(pub KeyCode);

pub fn on_btn_pressed(
    mut cmd: Commands,
    q_btns: Query<(Entity, &Interaction, &Callback), Changed<Interaction>>,
) {
    for (entity, interaction, callback) in q_btns.iter() {
        if matches!(interaction, Interaction::Pressed) {
            cmd.entity(entity).insert(Triggered);
            cmd.run_system(callback.0);
            cmd.entity(entity).remove::<Triggered>();
        }
    }
}

pub fn on_key_pressed(
    mut cmd: Commands,
    q_hotkeys: Query<(Entity, &Hotkey, &Callback)>,
    keys: Res<KeyInput>,
) {
    for (entity, hotkey, callback) in q_hotkeys.iter() {
        if keys.is_pressed(hotkey.0) {
            cmd.entity(entity).insert(Triggered);
            cmd.run_system(callback.0);
            cmd.entity(entity).remove::<Triggered>();
        }
    }
}
