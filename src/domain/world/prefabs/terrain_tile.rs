use super::{PrefabId, SpawnConfig};
use crate::{
    domain::{ApplyVisibilityEffects, TerrainNoise, ZoneStatus},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_terrain_tile(entity: Entity, world: &mut World, config: SpawnConfig) {
    let terrain = match config.prefab_id {
        PrefabId::TerrainTile(terrain) => terrain,
        _ => panic!("spawn_terrain_tile called with non-TerrainTile prefab_id"),
    };

    let Some(mut terrain_noise) = world.get_resource_mut::<TerrainNoise>() else {
        panic!("doh!");
    };

    let (x, y, _) = config.pos;

    let style = terrain_noise.style(terrain, (x, y));

    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        terrain,
        Glyph::new_from_style(style).layer(Layer::Terrain),
        ApplyVisibilityEffects,
        ZoneStatus::Dormant,
        RecordZonePosition,
        CleanupStatePlay,
    ));
}
