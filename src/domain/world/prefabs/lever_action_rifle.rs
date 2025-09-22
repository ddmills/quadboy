use super::{Prefab, PrefabBuilder, generate_weapon_from_prefab};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, Weapon},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_lever_action_rifle(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    // Generate weapon with potential rarity modifiers
    let generated_weapon = generate_weapon_from_prefab(
        &config,
        Weapon::rifle(),
        "Lever-action Rifle",
        "Precision machinery worn smooth by killing hands. Distance makes cowards of us all.",
    );

    let builder = PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking() // Items on ground don't move
        .with_glyph(203, Palette::Gray, Palette::Brown, Layer::Objects)
        .with_label(&generated_weapon.name)
        .with_description(&generated_weapon.description)
        .with_item(4.0)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::BothHands],
            EquipmentType::Weapon,
        ))
        .with_weapon(generated_weapon.weapon)
        .with_needs_stable_id();

    // Add the rarity component
    world.entity_mut(entity).insert(generated_weapon.rarity);

    builder
}
