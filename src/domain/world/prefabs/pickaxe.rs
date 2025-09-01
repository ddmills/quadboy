use super::Prefab;
use crate::{
    common::Palette,
    domain::{ApplyVisibilityEffects, Item, Label, NeedsStableId, SaveFlag},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_pickaxe(entity: Entity, world: &mut World, config: Prefab) {
    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(23, Palette::Brown, Palette::Gray).layer(Layer::Objects),
        Label::new("Pickaxe"),
        Item::new(2.0),
        ApplyVisibilityEffects,
        RecordZonePosition,
        SaveFlag,
        NeedsStableId,
        CleanupStatePlay,
    ));
}
