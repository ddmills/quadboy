use super::{Prefab, PrefabBuilder};
use crate::{
    common::{Palette, Rand},
    domain::{EquipmentSlot, EquipmentType, Equippable, StatModifier, StatModifiers, StatType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_ring(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    let mut rand = world.resource_mut::<Rand>();
    let stats = StatType::all();
    let random_stat = rand.pick(stats);

    let mut stat_modifiers = StatModifiers::new();
    stat_modifiers.add_modifier(
        random_stat,
        StatModifier::intrinsic(1, format!("Ring of {}", random_stat.verb())),
    );

    let label = format!("Ring of {}", random_stat.verb());

    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking() // Items on ground don't move
        .with_glyph(43, Palette::Yellow, Palette::Red, Layer::Objects)
        .with_label(&label)
        .with_description(
            "Simple band of tarnished metal. Promises and curses wear the same weight.",
        )
        .with_item(0.05)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::Ring1, EquipmentSlot::Ring2],
            EquipmentType::Accessory,
        ))
        .with_stat_modifiers(stat_modifiers)
        .with_needs_stable_id()
        
}
