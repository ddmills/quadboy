use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{Attributes, DefaultMeleeAttack, LootDrop, LootTableId, StatModifier, StatModifiers, StatType, Stats},
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_rattlesnake(entity: Entity, world: &mut World, config: Prefab) {
    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        StatType::Dodge,
        StatModifier::intrinsic(4, "Slippery".to_string()),
    );

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
        .with_health()
        .with_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::venomous_bite())
        .with_level(3)
        .with_attributes(Attributes::new(1, 4, 2, 2))
        .with_stats(Stats::new())
        .with_stat_modifiers(stat_modifiers)
        .with_loot_drop(LootDrop::new(LootTableId::RattlesnakeLoot, 0.4))
        .build();
}
