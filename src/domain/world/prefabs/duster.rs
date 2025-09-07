use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_duster(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(52, Palette::Brown, Palette::Gray, Layer::Objects)
        .with_label("Duster")
        .with_item(1.5)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Body],
            EquipmentType::Armor,
        ))
        .with_needs_stable_id()
        .build();
}
