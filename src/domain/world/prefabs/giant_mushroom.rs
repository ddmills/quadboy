use bevy_ecs::prelude::*;

use super::PrefabBuilder;
use crate::{
    common::{Palette, Rand},
    domain::{LightSource, MaterialType, Prefab},
    rendering::Layer,
};

pub fn spawn_giant_mushroom(entity: Entity, world: &mut World, config: Prefab) {
    let glyph_char = {
        let mut rand = world.get_resource_mut::<Rand>().unwrap();
        rand.pick(&[78, 79])
    };

    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_static_tracking() // Giant mushrooms never move
        .with_needs_stable_id()
        .with_glyph(glyph_char, Palette::White, Palette::Red, Layer::Objects)
        .with_label("{R|G}iant {R|M}ushroom")
        .with_description(
            "Pale and bloated in the underground dark. Fed on things better left buried.",
        )
        .with_collider()
        .with_vision_blocker()
        .with_light_source(LightSource::mushroom())
        .with_destructible(5, MaterialType::Wood)
        .build();
}
