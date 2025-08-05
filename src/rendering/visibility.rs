use bevy_ecs::prelude::*;
use macroquad::telemetry;

#[allow(dead_code)]
#[derive(Component, Default, Clone, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
}

#[derive(Component)]
pub struct IsVisible;

pub fn update_visibility(
    mut cmds: Commands,
    q_visibles: Query<(Entity, &Visibility), Changed<Visibility>>,
) {
    telemetry::begin_zone("update_visibility");

    for (entity, visibility) in q_visibles.iter() {
        if *visibility == Visibility::Visible {
            cmds.entity(entity).try_insert(IsVisible);
        } else {
            cmds.entity(entity).try_remove::<IsVisible>();
        }
    }

    telemetry::end_zone();
}
