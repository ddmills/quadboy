use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{LootDrop, LootTableId},
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_brown_bear(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph_and_texture(
            36,
            Palette::Brown,
            Palette::DarkBrown,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("{X|Brown Bear}")
        .with_energy(-150)
        .with_health(25)
        .with_collider()
        .with_hide_when_not_visible()
        .with_loot_drop(LootDrop::new(LootTableId::BrownBearLoot, 0.3))
        .build();
}
