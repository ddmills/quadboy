use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{DefaultMeleeAttack, LootDrop, LootTableId},
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_bat(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph_and_texture(
            34,
            Palette::DarkGray,
            Palette::Black,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("Bat")
        .with_energy(-80)
        .with_health(8)
        .with_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::wing_buffet())
        .with_level(1)
        .with_attributes(crate::domain::Attributes::new(1, 5, 1, 3))
        .with_stats(crate::domain::Stats::new())
        .with_stat_modifiers(crate::domain::StatModifiers::new())
        .with_loot_drop(LootDrop::new(LootTableId::BatLoot, 0.2))
        .build();
}
