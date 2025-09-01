use super::Prefab;
use crate::{
    common::Palette,
    domain::{ApplyVisibilityEffects, Collider, Inventory, Label, SaveFlag},
    engine::assign_stable_id,
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_chest(entity: Entity, world: &mut World, config: Prefab) {
    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(125, Palette::Brown, Palette::Yellow).layer(Layer::Objects),
        Label::new("Chest"),
        Inventory::new(20), // Chest can hold 20 items
        Collider,
        ApplyVisibilityEffects,
        RecordZonePosition,
        SaveFlag,
        CleanupStatePlay,
    ));

    assign_stable_id(entity, world);
}
