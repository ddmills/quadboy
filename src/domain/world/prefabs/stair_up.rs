use super::SpawnConfig;
use crate::{
    common::Palette,
    domain::{ApplyVisibilityEffects, Label, SaveFlag, StairUp},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_stair_up(entity: Entity, world: &mut World, config: SpawnConfig) {
    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(108, Palette::White, Palette::Clear).layer(Layer::Objects),
        Label::new("Stairs Up"),
        StairUp,
        RecordZonePosition,
        ApplyVisibilityEffects,
        SaveFlag,
        CleanupStatePlay,
    ));
}
