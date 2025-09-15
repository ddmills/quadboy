use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{ConsumableEffect, StackableType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_apple(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(41, Palette::Red, Palette::Green, Layer::Objects)
        .with_label("{R|Apple}")
        .with_description(
            "Wrinkled and bitter-sweet. Someone planted these trees long ago, before the troubles.",
        )
        .with_item(0.2)
        .with_needs_stable_id()
        .with_stackable(StackableType::Apple, 1)
        .with_consumable(ConsumableEffect::Heal(2), true)
        .build();
}
