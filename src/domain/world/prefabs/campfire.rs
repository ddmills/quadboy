use super::{Prefab, PrefabBuilder};
use crate::{common::Palette, domain::LightSource, rendering::Layer};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_campfire(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_static_tracking() // Campfires never move
        .with_needs_stable_id()
        .with_animated_glyph(
            vec![36, 37, 38],
            4.0,
            Palette::Red,
            Palette::Yellow,
            Layer::Objects,
            true,
        )
        .with_label("{R|C}ampfire")
        .with_description("Ashes and char, ringed with stones like broken teeth. The only honest light in a dishonest land.")
        .with_light_source(LightSource::campfire())
        .build();
}
