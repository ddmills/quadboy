use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{
        FactionId, FactionMember, LightSource, LootDrop, LootTableId,
        components::ai_controller::{AiController, AiTemplate},
    },
    rendering::{GlyphTextureId, Layer, Position},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_giant_firefly(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_needs_stable_id()
        .with_glyph_and_texture(
            38,
            Palette::Yellow,
            Palette::Orange,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("Giant Firefly")
        .with_description(
            "Swollen with unnatural light, drifting through the dark like fevered dreams.",
        )
        .with_energy(-110)
        .with_health()
        .with_collider()
        .with_hide_when_not_visible()
        .with_level(2)
        .with_attributes(crate::domain::Attributes::new(1, 3, 2, 2))
        .with_stats(crate::domain::Stats::new())
        .with_stat_modifiers(crate::domain::StatModifiers::new())
        .with_light_source(LightSource::new(0.6, 0xC4D434, 3).with_flicker(0.5))
        .with_loot_drop(LootDrop::new(LootTableId::GiantFireflyLoot, 0.35))
        .with_component(
            AiController::new(
                AiTemplate::Timid,
                Position::new(config.pos.0, config.pos.1, config.pos.2),
            )
            .with_ranges(30.0, 8.0, 12.0), // Doubled leash range
        )
        .with_component(FactionMember::new(FactionId::Wildlife))
        .build();
}
