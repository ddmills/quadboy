use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{
        CreatureType, DefaultMeleeAttack, FactionId, FactionMember, LootDrop, LootTableId,
        StatModifier, StatModifiers, StatType,
        components::ai_controller::{AiController, AiTemplate},
    },
    rendering::{GlyphTextureId, Layer, Position},
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
        .with_needs_stable_id()
        .with_glyph_and_texture(
            11,
            Palette::Red,
            Palette::White,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("{R|Bandit}")
        .with_description(
            "Eyes like burnt coal and a soul to match. The desert makes men into something else.",
        )
        .with_energy(-100)
        .with_health()
        .with_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::fists())
        .with_level(4)
        .with_attributes(crate::domain::Attributes::new(3, 3, 2, 2))
        .with_stats(crate::domain::Stats::new())
        .with_stat_modifiers(stat_modifiers)
        .with_loot_drop(LootDrop::new(LootTableId::BanditLoot, 0.5))
        .with_creature_type(CreatureType::Bandit)
        .with_component(
            AiController::new(
                AiTemplate::BasicAggressive,
                Position::new(config.pos.0, config.pos.1, config.pos.2),
            )
            .with_ranges(40.0, 10.0, 15.0),
        )
        .with_component(FactionMember::new(FactionId::Bandits))
        .build();
}
