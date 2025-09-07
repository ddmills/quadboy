use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    engine::AudioKey,
    rendering::{Layer, Text, text_content_length},
    ui::{Activatable, ActivatableBuilder, Callback, Focusable, Hotkey, Interactable, Interaction},
};

// Button is now just an alias for Activatable
// This allows backward compatibility during migration
pub type Button = Activatable;

// Compatibility constructor functions
impl Activatable {
    pub fn new<S: Into<String>>(label: S, callback: SystemId) -> Activatable {
        ActivatableBuilder::new(&label.into(), callback).as_button(Layer::Ui)
    }

    pub fn layer(self, layer: Layer) -> Activatable {
        // If this is already a button, recreate with new layer
        if let Some((label, _)) = self.as_button() {
            ActivatableBuilder::new(label, self.callback())
                .with_hotkeys(self.hotkeys())
                .with_audio(self.audio_key().unwrap_or(AudioKey::Button1))
                .as_button(layer)
        } else {
            // If not a button, just return self (shouldn't happen in normal use)
            self
        }
    }

    pub fn hotkey(self, key: KeyCode) -> Activatable {
        match self {
            Activatable::Button {
                label,
                callback,
                mut hotkeys,
                audio_key,
                layer,
                hover_color,
                pressed_color,
                normal_color,
                focus_order,
            } => {
                hotkeys.push(key);
                Activatable::Button {
                    label,
                    callback,
                    hotkeys,
                    audio_key,
                    layer,
                    hover_color,
                    pressed_color,
                    normal_color,
                    focus_order,
                }
            }
            _ => self, // Not a button, return as-is
        }
    }

    pub fn with_audio(self, audio_key: AudioKey) -> Activatable {
        match self {
            Activatable::Button {
                label,
                callback,
                hotkeys,
                audio_key: _,
                layer,
                hover_color,
                pressed_color,
                normal_color,
                focus_order,
            } => Activatable::Button {
                label,
                callback,
                hotkeys,
                audio_key: Some(audio_key),
                layer,
                hover_color,
                pressed_color,
                normal_color,
                focus_order,
            },
            _ => self, // Not a button, return as-is
        }
    }
}

pub fn setup_buttons(
    mut cmds: Commands,
    mut q_buttons: Query<
        (
            Entity,
            &Activatable,
            Option<&mut Text>,
            Option<&Callback>,
            Option<&Hotkey>,
            Option<&Interaction>,
        ),
        (Changed<Activatable>, With<Activatable>),
    >,
) {
    for (entity, activatable, text_opt, callback_opt, hotkey_opt, interaction_opt) in
        q_buttons.iter_mut()
    {
        let mut ecmds = cmds.entity(entity);

        // Add Focusable component for all activatable elements
        let focusable = if let Some(order) = activatable.focus_order() {
            Focusable::new().with_order(order)
        } else {
            Focusable::new()
        };
        ecmds.insert(focusable);

        if interaction_opt.is_none() {
            ecmds.insert(Interaction::None);
        }

        if let Some((label, layer)) = activatable.as_button() {
            let len = text_content_length(label);
            ecmds.insert(Interactable::new(len as f32 / 2., 0.5));

            if let Some(mut text) = text_opt {
                if text.value != label {
                    text.value = label.to_string();
                }
            } else {
                ecmds.insert(Text::new(label).layer(layer));
            }
        }

        if let Some(callback) = callback_opt {
            if callback.0 != activatable.callback() {
                ecmds.insert(Callback(activatable.callback()));
            }
        } else {
            ecmds.insert(Callback(activatable.callback()));
        }

        if let Some(first_hotkey) = activatable.hotkeys().first() {
            if let Some(hotkey) = hotkey_opt {
                if hotkey.0 != *first_hotkey {
                    ecmds.insert(Hotkey(*first_hotkey));
                }
            } else {
                ecmds.insert(Hotkey(*first_hotkey));
            }
        } else if hotkey_opt.is_some() {
            ecmds.remove::<Hotkey>();
        }
    }
}
