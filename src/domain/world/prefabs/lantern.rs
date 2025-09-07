use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable, LightSource},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_lantern(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(22, Palette::Gray, Palette::Yellow, Layer::Objects)
        .with_label("Lantern")
        .with_item(1.0)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::OffHand],
            EquipmentType::Tool,
        ))
        .with_light_source(LightSource::lantern())
        .with_lightable()
        .with_needs_stable_id()
        .build();
}
