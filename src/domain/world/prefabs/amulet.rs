use super::{Prefab, PrefabBuilder};
use crate::{
    common::{Palette, Rand},
    domain::{EquipmentSlot, EquipmentType, Equippable, StatModifier, StatModifiers, StatType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_amulet(entity: Entity, world: &mut World, config: Prefab) {
    let mut rand = world.resource_mut::<Rand>();
    let stats = StatType::all();
    let random_stat = rand.pick(stats);

    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        random_stat,
        StatModifier::intrinsic(1, format!("Amulet of {}", random_stat.verb())),
    );

    let label = format!("Amulet of {}", random_stat.verb());

    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_static_tracking() // Items on ground don't move
        .with_glyph(42, Palette::Yellow, Palette::Purple, Layer::Objects)
        .with_label(&label)
        .with_description(
            "Worn smooth by desperate fingers. Whatever power it held has long since fled.",
        )
        .with_item(0.1)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Neck],
            EquipmentType::Accessory,
        ))
        .with_stat_modifiers(stat_modifiers)
        .with_needs_stable_id()
        .build();
}
