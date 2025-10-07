use super::{Prefab, PrefabBuilder, SpawnValue};
use crate::common::Rand;
use crate::{
    common::Palette,
    domain::{ColliderFlags, MaterialType},
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_tree(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    let glyph_char = {
        let mut rand = world.get_resource_mut::<Rand>().unwrap();
        rand.pick(&[80, 81, 82])
    };

    let fg1 = if let Some(SpawnValue::Palette(color)) = config.metadata.get("fg1") {
        *color
    } else {
        Palette::DarkCyan
    };

    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_static_tracking()
        .with_needs_stable_id()
        .with_glyph(glyph_char, fg1, Palette::Brown, Layer::Objects)
        .with_label("{c|T}ree")
        .with_description("A sturdy tree with thick branches and deep roots.")
        .with_collider_flags(ColliderFlags::WALL)
        .with_destructible(5, MaterialType::Wood)
        .with_light_blocker()
}
