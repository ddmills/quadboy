use super::{Prefab, PrefabBuilder};
use crate::common::Rand;
use crate::domain::MaterialType;
use crate::{common::Palette, rendering::Layer};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_cactus(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    let glyph_idx = {
        let mut rand = world.get_resource_mut::<Rand>().unwrap();
        rand.pick(&[67, 68])
    };

    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking() // Cacti never move
        .with_needs_stable_id()
        .with_glyph(glyph_idx, Palette::Green, Palette::Purple, Layer::Objects)
        .with_label("Cactus")
        .with_description(
            "Twisted flesh armored in thorns. Patience made plant, waiting decades for rain.",
        )
        .with_collider()
        .with_vision_blocker()
        .with_destructible(10, MaterialType::Wood)
        
}
