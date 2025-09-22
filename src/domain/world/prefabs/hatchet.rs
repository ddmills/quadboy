use super::{Prefab, PrefabBuilder, generate_weapon_from_prefab};
use crate::{
    common::Palette,
    domain::{Equippable, Weapon},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_hatchet(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    // Generate weapon with potential rarity modifiers
    let generated_weapon = generate_weapon_from_prefab(
        &config,
        Weapon::hatchet(),
        "Hatchet",
        "Worn handle stained with pine sap and worse. Sharp enough for kindling or bone.",
    );

    let builder = PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking()
        .with_glyph(21, Palette::Brown, Palette::Gray, Layer::Objects)
        .with_label(&generated_weapon.name)
        .with_description(&generated_weapon.description)
        .with_item(2.0)
        .with_equippable(Equippable::tool())
        .with_weapon(generated_weapon.weapon)
        .with_needs_stable_id();

    // Add the rarity component
    world.entity_mut(entity).insert(generated_weapon.rarity);

    builder
}
