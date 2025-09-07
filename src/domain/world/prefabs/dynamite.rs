use super::{Prefab, PrefabBuilder};
use crate::{common::Palette, domain::StackableType, rendering::Layer};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_dynamite(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(25, Palette::Red, Palette::White, Layer::Objects)
        .with_label("Dynamite")
        .with_item(0.5)
        .with_needs_stable_id()
        .with_stackable(StackableType::Dynamite, 1)
        .build();
}
