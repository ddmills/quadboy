use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, StatModifier, StatModifiers, StatType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_poncho(entity: Entity, world: &mut World, config: Prefab) {
    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        StatType::Armor,
        StatModifier::intrinsic(2, "Weather Resistance".to_string()),
    );

    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_static_tracking() // Items on ground don't move
        .with_glyph(69, Palette::Yellow, Palette::Brown, Layer::Objects)
        .with_label("Poncho")
        .with_description("Faded patterns from another land. Rain runs off like tears on stone.")
        .with_item(1.0)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Body],
            EquipmentType::Armor,
        ))
        .with_stat_modifiers(stat_modifiers)
        .with_needs_stable_id()
        .build();
}
