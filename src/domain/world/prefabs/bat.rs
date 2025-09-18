use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{
        CreatureType, DefaultMeleeAttack, FactionId, FactionMember, LootDrop, LootTableId,
        components::ai_controller::{AiController, AiTemplate},
    },
    rendering::{GlyphTextureId, Layer, Position},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_bat(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_needs_stable_id()
        .with_glyph_and_texture(
            34,
            Palette::DarkGray,
            Palette::Black,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("Bat")
        .with_description(
            "Leather wings and needle teeth, shrieking in frequencies that make horses mad.",
        )
        .with_energy(-80)
        .with_health()
        .with_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::wing_buffet())
        .with_level(1)
        .with_attributes(crate::domain::Attributes::new(1, 5, 1, 3))
        .with_stats(crate::domain::Stats::new())
        .with_stat_modifiers(crate::domain::StatModifiers::new())
        .with_loot_drop(LootDrop::new(LootTableId::BatLoot, 0.2))
        .with_creature_type(CreatureType::Bat)
        .with_component(
            AiController::new(
                AiTemplate::BasicAggressive,
                Position::new(config.pos.0, config.pos.1, config.pos.2),
            )
            .with_ranges(30.0, 8.0, 12.0),
        )
        .with_component(FactionMember::new(FactionId::Wildlife))
        .build();
}
