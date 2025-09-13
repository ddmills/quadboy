use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{DefaultMeleeAttack, LightSource, LootDrop, LootTableId},
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_giant_firefly(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph_and_texture(
            38,
            Palette::Yellow,
            Palette::Orange,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("Giant Firefly")
        .with_energy(-110)
        .with_health(12)
        .with_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::electric_touch())
        .with_level(2)
        .with_attributes(crate::domain::Attributes::new(1, 3, 2, 2))
        .with_stats(crate::domain::Stats::new())
        .with_stat_modifiers(crate::domain::StatModifiers::new())
        .with_light_source(LightSource::new(0.6, 0xC4D434, 3).with_flicker(0.5))
        .with_loot_drop(LootDrop::new(LootTableId::GiantFireflyLoot, 0.35))
        .build();
}
