use super::{Prefab, PrefabBuilder};
use crate::common::Rand;
use crate::{common::Palette, domain::MaterialType, rendering::Layer};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_pine_tree(entity: Entity, world: &mut World, config: Prefab) {
    let glyph_char = {
        let mut rand = world.get_resource_mut::<Rand>().unwrap();
        rand.pick(&[45, 46, 47])
    };

    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_needs_stable_id()
        .with_glyph(
            glyph_char,
            Palette::DarkCyan,
            Palette::Brown,
            Layer::Objects,
        )
        .with_label("{c|P}ine {c|T}ree")
        .with_description("Scarred bark and sap like blood. Roots deep in soil that's tasted iron.")
        .with_collider()
        .with_destructible(5, MaterialType::Wood)
        .with_vision_blocker()
        .with_light_blocker()
        .build();
}
