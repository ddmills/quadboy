use super::{Prefab, PrefabBuilder};
use crate::{
    common::Palette,
    domain::{ExplosiveProperties, StackableType},
    engine::AudioKey,
    rendering::Layer,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_dynamite(entity: Entity, world: &mut World, config: Prefab) {
    PrefabBuilder::new(entity, world, &config)
        .with_base_components()
        .with_glyph(25, Palette::Red, Palette::White, Layer::Objects)
        .with_label("Dynamite")
        .with_description("Sweating nitroglycerin in the heat. The miner's prayer and last resort.")
        .with_item(0.5)
        .with_needs_stable_id()
        .with_stackable(StackableType::Dynamite, 1)
        .with_throwable_char(5, '!', Palette::Red.into())
        .with_component(ExplosiveProperties::dynamite())
        .with_lightable_audio(AudioKey::IgniteMatch, None)
        .build();
}
