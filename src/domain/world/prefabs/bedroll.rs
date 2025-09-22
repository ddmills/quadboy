use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{EquipmentSlot, EquipmentType, Equippable},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_bedroll(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking() // Items on ground don't move
        .with_glyph(54, Palette::Purple, Palette::Gray, Layer::Objects)
        .with_label("Bedroll")
        .with_description("Canvas and wool, stained with trail dust and darker things. Dreams come harder on the ground.")
        .with_item(2.0)
        .with_equippable(Equippable::new(
            vec![EquipmentSlot::OffHand],
            EquipmentType::Tool,
        ))
        .with_needs_stable_id()
        
}
