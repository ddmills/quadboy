use super::{Prefab, PrefabBuilder};
use crate::{common::Palette, domain::StackableType, rendering::Layer};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_gold_nugget(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(28, Palette::Yellow, Palette::White, Layer::Objects)
        .with_label("{Y-Y-Y-Y-Y-Y-Y-Y-Y-Y-Y-W scrollf|Gold Nugget}")
        .with_item(0.5)
        .with_needs_stable_id()
        .with_stackable(StackableType::GoldNugget, 1)
        .build();
}
