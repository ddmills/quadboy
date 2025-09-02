use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{BitmaskStyle, LootDrop, LootTableId, MaterialType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_boulder(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(68, Palette::Gray, Palette::Clear, Layer::Objects)
        .with_bitmask(BitmaskStyle::Wall)
        .with_label("Boulder")
        .with_collider()
        .with_destructible(10, MaterialType::Stone)
        .with_vision_blocker()
        .with_loot_drop(LootDrop::new(LootTableId::BoulderLoot, 0.25))
        .build();
}
