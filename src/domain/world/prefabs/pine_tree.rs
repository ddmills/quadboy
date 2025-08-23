use super::SpawnConfig;
use crate::common::Rand;
use crate::domain::Label;
use crate::{
    common::Palette,
    domain::{Collider, SaveFlag, ZoneStatus},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, hierarchy::ChildOf, world::World};

pub fn spawn_pine_tree(entity: Entity, world: &mut World, config: SpawnConfig) {
    let mut rand = world.get_resource_mut::<Rand>().unwrap();
    let glyph_char = rand.pick(&[64, 47]);

    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(glyph_char, Palette::DarkCyan, Palette::Red).layer(Layer::Objects),
        Label::new("Pine Tree"),
        Collider,
        RecordZonePosition,
        SaveFlag,
        CleanupStatePlay,
    ));
}
