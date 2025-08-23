use super::SpawnConfig;
use crate::{
    common::Palette,
    domain::{Label, SaveFlag, StairDown, ZoneStatus},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, hierarchy::ChildOf, world::World};

pub fn spawn_stair_down(entity: Entity, world: &mut World, config: SpawnConfig) {
    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(107, Palette::White, Palette::Clear).layer(Layer::Objects),
        Label::new("Stairs Down"),
        StairDown,
        ChildOf(config.zone_entity),
        ZoneStatus::Dormant,
        RecordZonePosition,
        SaveFlag,
        CleanupStatePlay,
    ));
}
