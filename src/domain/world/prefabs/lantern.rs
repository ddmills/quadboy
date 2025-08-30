use super::Prefab;
use crate::{
    common::Palette,
    domain::{ApplyVisibilityEffects, Label, SaveFlag},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_lantern(entity: Entity, world: &mut World, config: Prefab) {
    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(22, Palette::Gray, Palette::Yellow).layer(Layer::Objects),
        Label::new("Lantern"),
        ApplyVisibilityEffects,
        RecordZonePosition,
        SaveFlag,
        CleanupStatePlay,
    ));
}
