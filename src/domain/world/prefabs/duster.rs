use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, StatModifier, StatModifiers, StatType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_duster(entity: Entity, world: &mut World, config: Prefab) {
    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        StatType::Armor,
        StatModifier::intrinsic(4, "Thick Hide".to_string()),
    );

    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_static_tracking() // Items on ground don't move
        .with_glyph(52, Palette::Brown, Palette::Gray, Layer::Objects)
        .with_label("Duster")
        .with_description(
            "Trail-beaten canvas that's seen too many sunsets. Pockets full of dust and regret.",
        )
        .with_item(1.5)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Body],
            EquipmentType::Armor,
        ))
        .with_stat_modifiers(stat_modifiers)
        .with_needs_stable_id()
        .build();
}
