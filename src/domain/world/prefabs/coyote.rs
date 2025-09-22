use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{
        Attributes, CreatureType, DefaultMeleeAttack, FactionId, FactionMember, LootDrop,
        LootTableId,
        components::ai_controller::{AiController, AiTemplate},
    },
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_coyote(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_dynamic_tracking() // Coyotes can move
        .with_needs_stable_id()
        .with_glyph_and_texture(
            32,
            Palette::DarkYellow,
            Palette::Brown,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("Coyote")
        .with_description("Mangy and lean, forever watching from the ridge line. They know death when they smell it.")
        .with_energy(-130)
        .with_health()
        .with_actor_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::bite())
        .with_level(2)
        .with_attributes(Attributes::new(1, 2, 1, 2))
        .with_stats(crate::domain::Stats::new())
        .with_stat_modifiers(crate::domain::StatModifiers::new())
        .with_loot_drop(LootDrop::new(LootTableId::CoyoteLoot, 0.3))
        .with_creature_type(CreatureType::Coyote)
        .with_component(AiController::new(AiTemplate::BasicAggressive, config.pos))
        .with_component(FactionMember::new(FactionId::Wildlife))
        .with_movement_capabilities(crate::domain::MovementFlags::TERRESTRIAL)
        .build();
}
