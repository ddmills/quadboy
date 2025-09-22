use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, StatModifier, StatModifiers, StatType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_wool_shirt(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        StatType::Armor,
        StatModifier::intrinsic(3, "Basic Protection".to_string()),
    );

    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking() // Items on ground don't move
        .with_glyph(71, Palette::Gray, Palette::White, Layer::Objects)
        .with_label("Wool Shirt")
        .with_description("Rough-spun and twice-mended. Smells of campfire smoke and old sweat.")
        .with_item(0.8)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Body],
            EquipmentType::Armor,
        ))
        .with_stat_modifiers(stat_modifiers)
        .with_needs_stable_id()
        
}
