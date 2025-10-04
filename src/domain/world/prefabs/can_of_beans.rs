use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{ConsumableEffect, StackableType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_can_of_beans(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking()
        .with_glyph(85, Palette::Gray, Palette::Orange, Layer::Objects)
        .with_label("Can of Beans")
        .with_description(
            "Dented tin packed with refried beans. Tastes like survival and desperation.",
        )
        .with_item(0.3)
        .with_needs_stable_id()
        .with_stackable(StackableType::CanOfBeans, 1)
        .with_consumable(ConsumableEffect::RestoreArmor(5), true)
}
