use bevy_ecs::{component::Component, entity::Entity, resource::Resource};

use crate::{common::Grid, domain::ZoneSaveData};

#[derive(Resource, Default)]
pub struct Zones {
    pub active: Vec<usize>,
    pub player: usize,
}

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

    pub fn to_save(&self) -> ZoneSaveData {
        ZoneSaveData { idx: self.idx }
    }
}
