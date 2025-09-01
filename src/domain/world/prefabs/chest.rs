use super::Prefab;
use crate::{
    common::Palette,
    domain::{
        ApplyVisibilityEffects, Collider, Inventory, InventoryAccessible, Label, NeedsStableId,
        SaveFlag,
    },
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_chest(entity: Entity, world: &mut World, config: Prefab) {
    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(125, Palette::Brown, Palette::Yellow).layer(Layer::Objects),
        Label::new("Chest"),
        Inventory::new(5),
        InventoryAccessible,
        Collider,
        ApplyVisibilityEffects,
        RecordZonePosition,
        SaveFlag,
        NeedsStableId,
        CleanupStatePlay,
    ));
}
