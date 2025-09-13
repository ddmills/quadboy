use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{LootDrop, LootTableId},
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_bandit(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph_and_texture(
            11,
            Palette::Red,
            Palette::White,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("{R|Bandit}")
        .with_energy(-100)
        .with_health(10)
        .with_collider()
        .with_hide_when_not_visible()
        .with_level(4)
        .with_attributes(crate::domain::Attributes::new(3, 3, 2, 2))
        .with_stats(crate::domain::Stats::new())
        .with_stat_modifiers(crate::domain::StatModifiers::new())
        .with_loot_drop(LootDrop::new(LootTableId::BanditLoot, 0.5))
        .build();
}
