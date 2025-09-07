use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    engine::{AudioKey, AudioRegistry, KeyInput, Mouse},
    ui::{Button, DialogContent, DialogState, Interaction},
};

#[derive(Component, Clone, Debug)]
pub struct Callback(pub SystemId);

#[derive(Component, Clone, Debug)]
pub struct Triggered;

#[derive(Component, Clone, Debug)]
pub struct Hotkey(pub KeyCode);

pub fn on_btn_pressed(
    mut cmd: Commands,
    mut mouse: ResMut<Mouse>,
    q_btns: Query<(Entity, &Interaction, &Callback), Changed<Interaction>>,
    q_buttons: Query<&Button>,
    q_dialog_content: Query<&DialogContent>,
    dialog_state: Res<DialogState>,
    audio: Res<AudioRegistry>,
) {
    for (entity, interaction, callback) in q_btns.iter() {
        if matches!(interaction, Interaction::Released) {
            // If a dialog is open, only allow buttons from dialog content
            if dialog_state.is_open && q_dialog_content.get(entity).is_err() {
                continue;
            }

            mouse.is_captured = true;

            // Get custom audio key or default to Button1
            let audio_key = q_buttons
                .get(entity)
                .ok()
                .and_then(|button| button.audio_key())
                .unwrap_or(AudioKey::Button1);

            audio.play(audio_key, 0.7);
            cmd.entity(entity).insert(Triggered);
            cmd.run_system(callback.0);
            cmd.entity(entity).remove::<Triggered>();
        }
    }
}

pub fn on_key_pressed(
    mut cmd: Commands,
    q_hotkeys: Query<(Entity, &Hotkey, &Callback)>,
    q_buttons: Query<&Button>,
    q_dialog_content: Query<&DialogContent>,
    keys: Res<KeyInput>,
    dialog_state: Res<DialogState>,
    audio: Res<AudioRegistry>,
) {
    for (entity, hotkey, callback) in q_hotkeys.iter() {
        if keys.is_pressed(hotkey.0) {
            // If a dialog is open, only allow hotkeys from dialog content
            if dialog_state.is_open && q_dialog_content.get(entity).is_err() {
                continue; // Skip this hotkey since it's not part of a dialog
            }

            // Get custom audio key or default to Button1
            let audio_key = q_buttons
                .get(entity)
                .ok()
                .and_then(|button| button.audio_key())
                .unwrap_or(AudioKey::Button1);

            audio.play(audio_key, 0.7);
            cmd.entity(entity).insert(Triggered);
            cmd.run_system(callback.0);
            cmd.entity(entity).remove::<Triggered>();
        }
    }
}
