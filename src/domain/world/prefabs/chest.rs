use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_chest(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(125, Palette::Brown, Palette::Yellow, Layer::Objects)
        .with_label("Chest")
        .with_inventory(5)
        .with_inventory_accessible()
        .with_collider()
        .with_needs_stable_id()
        .build();
}
