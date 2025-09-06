use bevy_ecs::prelude::*;

use crate::{
    engine::Mouse,
    rendering::Position,
    ui::{DialogContent, DialogState},
};

#[derive(Component, Default, PartialEq, Eq)]
pub enum Interaction {
    Pressed,
    Released,
    Hovered,
    #[default]
    None,
}

#[derive(Component)]
#[require(Interaction)]
pub struct Interactable {
    pub width: f32,
    pub height: f32,
}

impl Interactable {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

pub fn ui_interaction_system(
    mut cmds: Commands,
    q_interactions: Query<(Entity, &Position, &Interactable, &Interaction)>,
    q_dialog_content: Query<&DialogContent>,
    mouse: Res<Mouse>,
    dialog_state: Res<DialogState>,
) {
    for (entity, position, interactable, current_interaction) in q_interactions.iter() {
        let mouse_ui = mouse.ui;
        let pos = (position.x, position.y);

        let is_hovered = mouse_ui.0 >= pos.0
            && mouse_ui.0 <= pos.0 + interactable.width
            && mouse_ui.1 > pos.1
            && mouse_ui.1 < pos.1 + interactable.height;

        let new_interaction = if is_hovered {
            if dialog_state.is_open && q_dialog_content.get(entity).is_err() {
                Interaction::None
            } else if mouse.left_pressed {
                Interaction::Pressed
            } else if mouse.left_just_released && !mouse.is_captured {
                Interaction::Released
            } else {
                Interaction::Hovered
            }
        } else {
            Interaction::None
        };

        if *current_interaction != new_interaction
            && let Ok(mut entity_cmds) = cmds.get_entity(entity) {
                entity_cmds.try_insert(new_interaction);
            }
    }
}
