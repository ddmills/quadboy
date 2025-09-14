use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, Weapon},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_double_barrel_shotgun(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(204, Palette::Gray, Palette::Brown, Layer::Objects)
        .with_label("Double-barrel Shotgun")
        .with_item(3.5)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::BothHands],
            EquipmentType::Weapon,
        ))
        .with_weapon(Weapon::shotgun())
        .with_needs_stable_id()
        .build();
}
