use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    rendering::{Layer, Text, text_content_length},
    ui::{Callback, Hotkey, Interactable, Interaction},
};

#[derive(Component)]
pub struct Button {
    pub label: String,
    pub hotkey: Option<KeyCode>,
    pub layer: Layer,
    pub callback: SystemId,
}

impl Button {
    pub fn new<S: Into<String>>(label: S, callback: SystemId) -> Self {
        Self {
            label: label.into(),
            hotkey: None,
            layer: Layer::Ui,
            callback,
        }
    }

    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    pub fn hotkey(mut self, key: KeyCode) -> Self {
        self.hotkey = Some(key);
        self
    }
}

pub fn setup_buttons(
    mut cmds: Commands,
    mut q_buttons: Query<
        (
            Entity,
            &Button,
            Option<&mut Text>,
            Option<&Callback>,
            Option<&Hotkey>,
            Option<&Interaction>,
        ),
        Changed<Button>,
    >,
) {
    for (entity, btn, text_opt, callback_opt, hotkey_opt, interaction_opt) in q_buttons.iter_mut() {
        let len = text_content_length(&btn.label);

        let mut ecmds = cmds.entity(entity);

        ecmds.insert(Interactable::new(len as f32 / 2., 0.5));

        if interaction_opt.is_none() {
            ecmds.insert(Interaction::None);
        }

        if let Some(mut text) = text_opt {
            if text.value != btn.label {
                text.value = btn.label.clone();
            }
        } else {
            ecmds.insert(Text::new(&btn.label).layer(btn.layer));
        }

        if let Some(callback) = callback_opt {
            if callback.0 != btn.callback {
                ecmds.insert(Callback(btn.callback));
            }
        } else {
            ecmds.insert(Callback(btn.callback));
        }

        if let Some(key) = btn.hotkey {
            if let Some(hotkey) = hotkey_opt {
                if hotkey.0 != key {
                    ecmds.insert(Hotkey(key));
                }
            } else {
                ecmds.insert(Hotkey(key));
            }
        } else if hotkey_opt.is_some() {
            ecmds.remove::<Hotkey>();
        }
    }
}

pub fn button_styles(mut q_buttons: Query<(&mut Text, &Button, &Interaction)>) {
    for (mut text, _button, interaction) in q_buttons.iter_mut() {
        let bg = match interaction {
            Interaction::Released => Palette::DarkBlue,
            Interaction::Pressed => Palette::DarkBlue,
            Interaction::Hovered => Palette::Gray,
            Interaction::None => Palette::Black,
        };

        text.bg = Some(bg.into());
    }
}
