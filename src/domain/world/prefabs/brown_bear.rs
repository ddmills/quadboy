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

pub fn spawn_brown_bear(entity: Entity, world: &mut World, config: Prefab) {
    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        StatType::Armor,
        StatModifier::intrinsic(10, "Thick Hide".to_string()),
    );
    stat_modifiers.add_modifier(
        StatType::ArmorRegen,
        StatModifier::intrinsic(3, "Natural Healing".to_string()),
    );

    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_needs_stable_id()
        .with_glyph_and_texture(
            36,
            Palette::Brown,
            Palette::DarkBrown,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("{X|Brown Bear}")
        .with_description("Eight hundred pounds of primal rage and hunger. The old gods still walk these mountains.")
        .with_energy(-150)
        .with_health()
        .with_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::claw_swipe())
        .with_level(4)
        .with_attributes(crate::domain::Attributes::new(4, 1, 3, 1))
        .with_stats(crate::domain::Stats::new())
        .with_stat_modifiers(stat_modifiers)
        .with_loot_drop(LootDrop::new(LootTableId::BrownBearLoot, 0.3))
        .with_creature_type(CreatureType::Bear)
        .with_component(AiController::new(AiTemplate::BasicAggressive, Position::new(config.pos.0, config.pos.1, config.pos.2))
            .with_ranges(80.0, 15.0, 25.0))
        .with_component(FactionMember::new(FactionId::Wildlife))
        .with_movement_capabilities(crate::domain::MovementFlags::TERRESTRIAL)
        .build();
}
