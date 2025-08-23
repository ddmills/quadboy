use super::SpawnConfig;
use crate::common::Rand;
use crate::domain::{ApplyVisibilityEffects, Label, VisionBlocker};
use crate::{
    common::Palette,
    domain::{Collider, SaveFlag},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_cactus(entity: Entity, world: &mut World, config: SpawnConfig) {
    let mut rand = world.get_resource_mut::<Rand>().unwrap();
    let glyph_idx = rand.pick(&[67, 68]);

    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::idx(glyph_idx)
            .fg1(Palette::Green)
            .fg2(Palette::Purple)
            .layer(Layer::Objects),
        Label::new("Cactus"),
        Collider,
        RecordZonePosition,
        ApplyVisibilityEffects,
        VisionBlocker,
        SaveFlag,
        CleanupStatePlay,
    ));
}
