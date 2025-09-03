use bevy_ecs::prelude::*;

use crate::{engine::Mouse, rendering::Position};

#[derive(Component, Default, PartialEq, Eq)]
pub enum Interaction {
    Pressed,
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
    mouse: Res<Mouse>,
) {
    for (entity, position, interactable, current_interaction) in q_interactions.iter() {
        let mouse_ui = mouse.ui;
        let pos = (position.x, position.y);

        let is_hovered = mouse_ui.0 >= pos.0
            && mouse_ui.0 <= pos.0 + interactable.width
            && mouse_ui.1 > pos.1
            && mouse_ui.1 < pos.1 + interactable.height;

        let new_interaction = if is_hovered {
            if mouse.left_pressed {
                Interaction::Pressed
            } else {
                Interaction::Hovered
            }
        } else {
            Interaction::None
        };

        if *current_interaction != new_interaction {
            cmds.entity(entity).insert(new_interaction);
        }
    }
}
