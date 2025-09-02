use bevy_ecs::prelude::*;

use super::PrefabBuilder;
use crate::{
    common::Palette,
    domain::{MaterialType, Prefab},
    rendering::Layer,
};

pub fn spawn_giant_mushroom(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(79, Palette::White, Palette::Red, Layer::Objects)
        .with_label("{R|G}iant {R|M}ushroom")
        .with_collider()
        .with_vision_blocker()
        .with_destructible(5, MaterialType::Wood)
        .build();
}
