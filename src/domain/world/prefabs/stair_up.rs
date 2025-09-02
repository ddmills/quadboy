use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_stair_up(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(108, Palette::White, Palette::Clear, Layer::Objects)
        .with_label("Stairs Up")
        .with_stair_up()
        .build();
}
