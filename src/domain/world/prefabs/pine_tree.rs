use super::Prefab;
use crate::common::Rand;
use crate::domain::{ApplyVisibilityEffects, Label};
use crate::{
    common::Palette,
    domain::{Collider, SaveFlag, VisionBlocker},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_pine_tree(entity: Entity, world: &mut World, config: Prefab) {
    let mut rand = world.get_resource_mut::<Rand>().unwrap();
    let glyph_char = rand.pick(&[45, 46, 47]);

    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(glyph_char, Palette::DarkCyan, Palette::Brown).layer(Layer::Objects),
        Label::new("{G|P}ine {G|T}ree"),
        Collider,
        VisionBlocker,
        RecordZonePosition,
        ApplyVisibilityEffects,
        SaveFlag,
        CleanupStatePlay,
    ));
}
