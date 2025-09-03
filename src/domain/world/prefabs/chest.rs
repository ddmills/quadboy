use super::{Prefab, PrefabBuilder, SpawnValue};
use crate::{common::Palette, domain::UnopenedContainer, rendering::Layer};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_chest(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(125, Palette::Brown, Palette::Yellow, Layer::Objects)
        .with_label("Chest")
        .with_inventory(25.0)
        .with_inventory_accessible()
        .with_needs_stable_id()
        .with_collider()
        .build();

    if let Some(SpawnValue::LootTableId(loot_table_id)) = config.metadata.get("loot_table_id") {
        world
            .entity_mut(entity)
            .insert(UnopenedContainer(*loot_table_id));
    }
}
