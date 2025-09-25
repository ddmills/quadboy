use super::{Prefab, PrefabBuilder, SpawnValue};
use crate::{common::Palette, domain::UnopenedContainer, engine::AudioKey, rendering::Layer};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_chest(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    let builder = PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking()
        .with_glyph(125, Palette::Brown, Palette::Yellow, Layer::Objects)
        .with_label("Chest")
        .with_description("Iron bands and old wood that remembers better days. Lock's been shot off more than once.")
        .with_inventory_audio(25.0, AudioKey::ChestOpen1)
        .with_inventory_accessible()
        .with_needs_stable_id()
        .with_collider();

    if let Some(SpawnValue::LootTableId(loot_table_id)) = config.metadata.get("loot_table_id") {
        world
            .entity_mut(entity)
            .insert(UnopenedContainer(*loot_table_id));
    }

    builder
}
