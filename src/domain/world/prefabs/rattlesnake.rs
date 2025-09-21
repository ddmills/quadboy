use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{
        Attributes, CreatureType, DefaultMeleeAttack, FactionId, FactionMember, LootDrop,
        LootTableId, StatModifier, StatModifiers, StatType, Stats,
        components::ai_controller::{AiController, AiTemplate},
    },
    rendering::{GlyphTextureId, Layer, Position},
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
        .with_needs_stable_id()
        .with_glyph_and_texture(
            33,
            Palette::DarkYellow,
            Palette::Yellow,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("{Y-y-X-y repeat|Rattlesnake}")
        .with_description(
            "Coiled malice baking in the sun. The desert's way of keeping honest men honest.",
        )
        .with_energy(-120)
        .with_health()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::venomous_bite())
        .with_level(3)
        .with_attributes(Attributes::new(1, 4, 2, 2))
        .with_stats(Stats::new())
        .with_stat_modifiers(stat_modifiers)
        .with_loot_drop(LootDrop::new(LootTableId::RattlesnakeLoot, 0.4))
        .with_creature_type(CreatureType::Rattlesnake)
        .with_component(AiController::new(AiTemplate::BasicAggressive, config.pos))
        .with_component(FactionMember::new(FactionId::Wildlife))
        .build();
}
