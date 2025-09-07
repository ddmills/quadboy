use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_steel_toe_boots(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(70, Palette::Brown, Palette::Gray, Layer::Objects)
        .with_label("{W-Y-W-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C scrollf|Steel-toe} Boots")
        .with_item(1.5)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Feet],
            EquipmentType::Armor,
        ))
        .with_needs_stable_id()
        .build();
}
