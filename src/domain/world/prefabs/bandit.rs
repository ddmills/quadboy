use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{LootDrop, LootTableId, StatModifier, StatModifiers, StatType},
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_bandit(entity: Entity, world: &mut World, config: Prefab) {
    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        StatType::Armor,
        StatModifier::intrinsic(2, "Leather Vest".to_string()),
    );

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
        .with_health()
        .with_collider()
        .with_hide_when_not_visible()
        .with_level(4)
        .with_attributes(crate::domain::Attributes::new(3, 3, 2, 2))
        .with_stats(crate::domain::Stats::new())
        .with_stat_modifiers(stat_modifiers)
        .with_loot_drop(LootDrop::new(LootTableId::BanditLoot, 0.5))
        .build();
}
