use super::Prefab;
use crate::{
    common::Palette,
    domain::{
        ApplyVisibilityEffects, Collider, Energy, Health, HideWhenNotVisible, Label, SaveFlag,
    },
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_bandit(entity: Entity, world: &mut World, config: Prefab) {
    let position = Position::new_world(config.pos);

    world.entity_mut(entity).insert((
        position,
        Glyph::new(145, Palette::Red, Palette::White).layer(Layer::Actors),
        Label::new("{R|Bandit}"),
        Energy::new(-100),
        Health::new(10),
        Collider,
        RecordZonePosition,
        ApplyVisibilityEffects,
        HideWhenNotVisible,
        SaveFlag,
        CleanupStatePlay,
    ));
}
