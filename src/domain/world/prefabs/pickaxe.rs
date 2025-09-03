use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{Equippable, MeleeWeapon},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_pickaxe(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(23, Palette::Brown, Palette::White, Layer::Objects)
        .with_label("Pickaxe")
        .with_item(2.0)
        .with_equippable(Equippable::tool())
        .with_melee_weapon(MeleeWeapon::pickaxe())
        .with_needs_stable_id()
        .build();
}
