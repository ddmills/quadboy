use super::{Prefab, PrefabBuilder, SpawnValue};
use crate::{
    common::Palette,
    domain::{BitmaskStyle, LootDrop, LootTableId, MaterialType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_boulder(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    let fg1 = if let Some(SpawnValue::Palette(color)) = config.metadata.get("fg1") {
        *color
    } else {
        Palette::Gray
    };

    let fg2 = if let Some(SpawnValue::Palette(color)) = config.metadata.get("fg2") {
        *color
    } else {
        Palette::Gray
    };

    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking() // Boulders never move
        .with_needs_stable_id()
        .with_glyph(68, fg1, fg2, Layer::Objects)
        .with_bitmask(BitmaskStyle::Rocks)
        .with_label("Boulder")
        .with_description(
            "Wind-carved monument to father time. Each crack a chronicle of forgotten years.",
        )
        .with_collider()
        .with_destructible(10, MaterialType::Stone)
        .with_vision_blocker()
        .with_light_blocker()
        .with_loot_drop(LootDrop::new(LootTableId::BoulderLoot, 0.25))
}
