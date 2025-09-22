use super::{Prefab, PrefabBuilder, generate_weapon_from_prefab};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, Weapon},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_navy_revolver(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    // Generate weapon with potential rarity modifiers
    let generated_weapon = generate_weapon_from_prefab(
        &config,
        Weapon::revolver(),
        "Navy Revolver",
        "Cold iron weight on the hip. Six arguments that don't require words.",
    );

    let builder = PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking() // Items on ground don't move
        .with_glyph(201, Palette::Gray, Palette::Brown, Layer::Objects)
        .with_label(&generated_weapon.name)
        .with_description(&generated_weapon.description)
        .with_item(1.5)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::MainHand],
            EquipmentType::Weapon,
        ))
        .with_weapon(generated_weapon.weapon)
        .with_needs_stable_id();

    // Add the rarity component
    world.entity_mut(entity).insert(generated_weapon.rarity);

    builder
}
