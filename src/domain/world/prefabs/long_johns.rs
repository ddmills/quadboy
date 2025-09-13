use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, StatModifier, StatModifiers, StatType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_long_johns(entity: Entity, world: &mut World, config: Prefab) {
    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        StatType::Speed,
        StatModifier::intrinsic(4, "Lightweight Comfort".to_string()),
    );

    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(53, Palette::White, Palette::Gray, Layer::Objects)
        .with_label("Long Johns")
        .with_item(0.5)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Legs],
            EquipmentType::Armor,
        ))
        .with_stat_modifiers(stat_modifiers)
        .with_needs_stable_id()
        .build();
}
