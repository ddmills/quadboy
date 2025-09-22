use super::{Prefab, PrefabBuilder, PrefabId};
use crate::{
    domain::{
        ApplyVisibilityEffects, StaticEntity, StaticEntitySpawnedEvent, TerrainNoise, ZoneStatus,
    },
    rendering::{Glyph, Layer, Position},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_terrain_tile(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    let terrain = match config.prefab_id {
        PrefabId::TerrainTile(terrain) => terrain,
        _ => panic!("spawn_terrain_tile called with non-TerrainTile prefab_id"),
    };

    let Some(mut terrain_noise) = world.get_resource_mut::<TerrainNoise>() else {
        panic!("doh!");
    };

    let (x, y, _) = config.pos;

    let style = terrain_noise.style(terrain, (x, y));

    let position = Position::new_world(config.pos);

    world.entity_mut(entity).insert((
        position.clone(),
        terrain,
        Glyph::new_from_style(style).layer(Layer::Terrain),
        ApplyVisibilityEffects,
        ZoneStatus::Dormant,
        StaticEntity, // Terrain tiles never move
        CleanupStatePlay,
    ));

    // Send event for static entity placement
    world.send_event(StaticEntitySpawnedEvent {
        entity,
        position,
        collider_flags: None, // Terrain tiles don't have colliders
    });

    // Return a dummy builder since this function handles everything manually
    PrefabBuilder::new()
}
