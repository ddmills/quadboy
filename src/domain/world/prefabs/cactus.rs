use super::{Prefab, PrefabBuilder};
use crate::common::Rand;
use crate::{
    common::Palette,
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_cactus(entity: Entity, world: &mut World, config: Prefab) {
    let glyph_idx = {
        let mut rand = world.get_resource_mut::<Rand>().unwrap();
        rand.pick(&[67, 68])
    };

    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(glyph_idx, Palette::Green, Palette::Purple, Layer::Objects)
        .with_label("Cactus")
        .with_collider()
        .with_vision_blocker()
        .build();
}
