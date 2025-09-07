use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, RangedWeapon},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_lever_action_rifle(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(203, Palette::Gray, Palette::Brown, Layer::Objects)
        .with_label("Lever-action Rifle")
        .with_item(4.0)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::BothHands],
            EquipmentType::Weapon,
        ))
        .with_ranged_weapon(RangedWeapon::rifle())
        .with_needs_stable_id()
        .build();
}