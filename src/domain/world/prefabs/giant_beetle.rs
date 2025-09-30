use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{
        Attributes, CreatureType, DefaultMeleeAttack, FactionId, FactionMember, LootDrop,
        LootTableId, StatModifier, StatModifiers, StatType, Stats,
        components::ai_controller::{AiController, AiTemplate},
    },
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_giant_beetle(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        StatType::Armor,
        StatModifier::intrinsic(18, "Chitin Plating".to_string()),
    );

    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_dynamic_tracking()
        .with_needs_stable_id()
        .with_glyph_and_texture(
            42,
            Palette::Purple,
            Palette::White,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("{G-g-X-g repeat|Giant Beetle}")
        .with_description(
            "A massive beetle with gleaming chitin armor. Its mandibles click menacingly as it scuttles across the desert sands.",
        )
        .with_energy(-100)
        .with_health()
        .with_actor_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::mandible_crush())
        .with_level(3)
        .with_attributes(Attributes::new(3, 2, 4, 1))
        .with_stats(Stats::new())
        .with_stat_modifiers(stat_modifiers)
        .with_loot_drop(LootDrop::new(LootTableId::BeetleLoot, 0.3))
        .with_creature_type(CreatureType::Beetle)
        .with_component(AiController::new(AiTemplate::BasicAggressive, config.pos))
        .with_component(FactionMember::new(FactionId::Wildlife))
}
