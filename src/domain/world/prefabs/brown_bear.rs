use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{DefaultMeleeAttack, LootDrop, LootTableId},
    rendering::{GlyphTextureId, Layer},
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_brown_bear(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph_and_texture(
            36,
            Palette::Brown,
            Palette::DarkBrown,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("{X|Brown Bear}")
        .with_energy(-150)
        .with_health(25)
        .with_collider()
        .with_hide_when_not_visible()
        .with_default_melee_attack(DefaultMeleeAttack::claw_swipe())
        .with_level(6)
        .with_loot_drop(LootDrop::new(LootTableId::BrownBearLoot, 0.3))
        .build();
}
