use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_long_johns(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(53, Palette::White, Palette::Gray, Layer::Objects)
        .with_label("Long Johns")
        .with_item(0.5)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Legs],
            EquipmentType::Armor,
        ))
        .with_needs_stable_id()
        .build();
}
