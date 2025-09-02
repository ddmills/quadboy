use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{Equippable, MeleeWeapon},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_hatchet(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(21, Palette::Red, Palette::Gray, Layer::Objects)
        .with_label("Hatchet")
        .with_item(2.0)
        .with_equippable(Equippable::tool())
        .with_melee_weapon(MeleeWeapon::hatchet())
        .with_needs_stable_id()
        .build();
}
