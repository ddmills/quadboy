use bevy_ecs::prelude::*;

use crate::{
    common::Palette,
    domain::{ApplyVisibilityEffects, Collider, Label, SaveFlag, SpawnConfig, VisionBlocker},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};

pub fn spawn_giant_mushroom(_entity: Entity, world: &mut World, config: SpawnConfig) {
    let entity = world.spawn_empty().id();

    let mut entity_mut = world.entity_mut(entity);

    entity_mut.insert((
        Position::new_world(config.pos),
        Glyph::new(79, Palette::White, Palette::Red).layer(Layer::Objects),
        Label::new("{R|G}iant {R|M}ushroom"),
        Collider,
        VisionBlocker,
        RecordZonePosition,
        ApplyVisibilityEffects,
        SaveFlag,
        CleanupStatePlay,
    ));
}
