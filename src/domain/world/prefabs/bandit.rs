use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{
        CreatureType, DefaultMeleeAttack, DefaultRangedAttack, FactionId, FactionMember, LootDrop,
        LootTableId, StatModifier, StatModifiers, StatType,
        components::ai_controller::{AiController, AiTemplate},
    },
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_bandit(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        StatType::Armor,
        StatModifier::intrinsic(2, "Leather Vest".to_string()),
    );

    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_dynamic_tracking() // Bandits can move
        .with_needs_stable_id()
        .with_glyph_and_texture(
            11,
            Palette::White,
            Palette::Red,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("Bandit")
        .with_description(
            "Eyes like burnt coal and a soul to match. The desert makes men into something else.",
        )
        .with_energy(-100)
        .with_health()
        .with_actor_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::fists())
        .with_component(DefaultRangedAttack::revolver())
        .with_level(4)
        .with_attributes(crate::domain::Attributes::new(3, 3, 2, 2))
        .with_stats(crate::domain::Stats::new())
        .with_stat_modifiers(stat_modifiers)
        .with_loot_drop(LootDrop::new(LootTableId::BanditLoot, 0.5))
        .with_creature_type(CreatureType::Bandit)
        .with_component(AiController::new(AiTemplate::BasicAggressive, config.pos))
        .with_component(FactionMember::new(FactionId::Bandits))
        .with_movement_capabilities(crate::domain::MovementFlags::TERRESTRIAL)
}
