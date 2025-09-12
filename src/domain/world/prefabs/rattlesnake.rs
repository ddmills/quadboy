use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{LootDrop, LootTableId},
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_rattlesnake(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph_and_texture(
            33,
            Palette::Green,
            Palette::DarkGreen,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("Rattlesnake")
        .with_energy(-120)
        .with_health(15)
        .with_collider()
        .with_hide_when_not_visible()
        .with_loot_drop(LootDrop::new(LootTableId::RattlesnakeLoot, 0.4))
        .build();
}
