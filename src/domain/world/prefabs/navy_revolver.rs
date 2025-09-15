use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, Weapon},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_navy_revolver(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(201, Palette::Gray, Palette::Brown, Layer::Objects)
        .with_label("Navy Revolver")
        .with_description("Cold iron weight on the hip. Six arguments that don't require words.")
        .with_item(1.5)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::MainHand],
            EquipmentType::Weapon,
        ))
        .with_weapon(Weapon::revolver())
        .with_needs_stable_id()
        .build();
}
