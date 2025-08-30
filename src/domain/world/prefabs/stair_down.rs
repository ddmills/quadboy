use super::Prefab;
use crate::{
    common::Palette,
    domain::{ApplyVisibilityEffects, Label, SaveFlag, StairDown},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_stair_down(entity: Entity, world: &mut World, config: Prefab) {
    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(107, Palette::White, Palette::Clear).layer(Layer::Objects),
        Label::new("Stairs Down"),
        StairDown,
        RecordZonePosition,
        ApplyVisibilityEffects,
        SaveFlag,
        CleanupStatePlay,
    ));
}
