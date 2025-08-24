use super::SpawnConfig;
use crate::{
    domain::{ApplyVisibilityEffects, ZoneStatus},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_terrain_tile(entity: Entity, world: &mut World, config: SpawnConfig) {
    let terrain = match config.prefab_id {
        super::PrefabId::TerrainTile(terrain) => terrain,
        _ => panic!("spawn_terrain_tile called with non-TerrainTile prefab_id"),
    };

    let idx = terrain.tile();
    let (bg, fg) = terrain.colors();

    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        terrain,
        Glyph::idx(idx).bg_opt(bg).fg1_opt(fg).layer(Layer::Terrain),
        ApplyVisibilityEffects,
        ZoneStatus::Dormant,
        RecordZonePosition,
        CleanupStatePlay,
    ));
}
