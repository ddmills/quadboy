use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{
        Attributes, CreatureType, DefaultMeleeAttack, FactionId, FactionMember, LootDrop,
        LootTableId, Stats,
        components::ai_controller::{AiController, AiTemplate},
    },
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_rat(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_dynamic_tracking() // Rats can move
        .with_needs_stable_id()
        .with_glyph_and_texture(
            44,
            Palette::Gray,
            Palette::DarkGray,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("Rat")
        .with_description(
            "Beady eyes and twitching whiskers. Where there's one, there's always more.",
        )
        .with_energy(-60)
        .with_health()
        .with_actor_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::nibble())
        .with_level(1)
        .with_attributes(Attributes::new(1, 4, 1, 1))
        .with_stats(Stats::new())
        .with_stat_modifiers(crate::domain::StatModifiers::new())
        .with_loot_drop(LootDrop::new(LootTableId::RatLoot, 0.1))
        .with_creature_type(CreatureType::Rat)
        .with_component(AiController::new(AiTemplate::BasicAggressive, config.pos))
        .with_component(FactionMember::new(FactionId::Wildlife))
}
