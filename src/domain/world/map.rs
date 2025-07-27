use bevy_ecs::{component::Component, entity::Entity, resource::Resource};

use crate::{cfg::MAP_SIZE, common::{Grid, Grid3d}};

#[derive(Resource)]
pub struct Map {
    pub zones: Grid3d<OverworldZone>
}

impl Default for Map {
    fn default() -> Self {
        Self {
            zones: Grid3d::init(MAP_SIZE.0, MAP_SIZE.1, MAP_SIZE.2, OverworldZone)
        }
    }
}

#[derive(Clone, Default)]
pub struct OverworldZone;

#[derive(Component)]
pub struct Zone {
    pub idx: usize,
    pub tiles: Grid<Entity>,
}

impl Zone {
    pub fn new(idx: usize, tiles: Grid<Entity>) -> Self
    {
        Self { idx, tiles }
    }
}
