use super::SpawnConfig;
use crate::{
    common::Palette,
    domain::{Collider, Energy, Label, SaveFlag, ZoneStatus},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, hierarchy::ChildOf, world::World};

pub fn spawn_bandit(entity: Entity, world: &mut World, config: SpawnConfig) {
    let position = Position::new_world(config.pos);

    world.entity_mut(entity).insert((
        position,
        Glyph::new(145, Palette::Red, Palette::White).layer(Layer::Actors),
        Label::new("{R|Bandit}"),
        Energy::new(-100),
        Collider,
        RecordZonePosition,
        SaveFlag,
        CleanupStatePlay,
    ));
}
