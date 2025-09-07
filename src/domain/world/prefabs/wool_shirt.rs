use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_wool_shirt(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(71, Palette::Gray, Palette::White, Layer::Objects)
        .with_label("Wool Shirt")
        .with_item(0.8)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Body],
            EquipmentType::Armor,
        ))
        .with_needs_stable_id()
        .build();
}
