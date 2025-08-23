use super::SpawnConfig;
use crate::{
    common::Palette,
    domain::{BitmaskGlyph, BitmaskStyle, Collider, Label, SaveFlag, ZoneStatus},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, hierarchy::ChildOf, world::World};

pub fn spawn_boulder(entity: Entity, world: &mut World, config: SpawnConfig) {
    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(68, Palette::Gray, Palette::Clear).layer(Layer::Objects),
        BitmaskGlyph::new(BitmaskStyle::Wall),
        Label::new("Boulder"),
        Collider,
        ChildOf(config.zone_entity),
        ZoneStatus::Dormant,
        RecordZonePosition,
        SaveFlag,
        CleanupStatePlay,
    ));
}
