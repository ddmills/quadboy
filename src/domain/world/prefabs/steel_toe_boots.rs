use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, StatModifier, StatModifiers, StatType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_steel_toe_boots(entity: Entity, world: &mut World, config: Prefab) {
    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        StatType::Fortitude,
        StatModifier::intrinsic(2, "Steel Protection".to_string()),
    );
    stat_modifiers.add_modifier(
        StatType::Armor,
        StatModifier::intrinsic(1, "Steel Protection".to_string()),
    );

    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_static_tracking() // Items on ground don't move
        .with_glyph(70, Palette::Brown, Palette::Gray, Layer::Objects)
        .with_label("{W-Y-W-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C scrollf|Steel-toe} Boots")
        .with_description(
            "Leather cracked like drought earth. Every scuff tells a story nobody wants to hear.",
        )
        .with_item(1.5)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Feet],
            EquipmentType::Armor,
        ))
        .with_stat_modifiers(stat_modifiers)
        .with_needs_stable_id()
        .build();
}
