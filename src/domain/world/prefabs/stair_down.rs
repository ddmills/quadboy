use super::{Prefab, PrefabBuilder};
use crate::{common::Palette, rendering::Layer};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_stair_down(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_static_tracking() // Stairs never move
        .with_needs_stable_id()
        .with_glyph(107, Palette::White, Palette::Clear, Layer::Objects)
        .with_label("Stairs Down")
        .with_description(
            "Carved by forgotten hands into living rock. Each step descends toward older darkness.",
        )
        .with_stair_down()
        .build();
}
