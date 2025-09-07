use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{Equippable, MeleeWeapon},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_cavalry_sword(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(20, Palette::Yellow, Palette::Gray, Layer::Objects)
        .with_label("Cavalry Sword")
        .with_item(3.0)
        .with_equippable(Equippable::weapon_one_handed())
        .with_melee_weapon(MeleeWeapon::sword())
        .with_needs_stable_id()
        .build();
}
