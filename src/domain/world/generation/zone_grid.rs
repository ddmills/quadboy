use crate::{
    cfg::ZONE_SIZE,
    common::Grid,
    domain::{Prefab, Terrain},
};

pub struct ZoneGridData {
    pub terrain: Grid<Terrain>,
    pub entities: Grid<Vec<Prefab>>,
    pub locked: Grid<bool>,
}

impl ZoneGridData {
    pub fn new(terrain: Terrain) -> Self {
        Self {
            terrain: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| terrain),
            entities: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| vec![]),
            locked: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| false),
        }
    }

    pub fn push_entity(&mut self, x: usize, y: usize, config: Prefab) {
        if let Some(ents) = self.entities.get_mut(x, y) {
            ents.push(config);
        }
    }


    pub fn set_terrain(&mut self, x: usize, y: usize, terrain: Terrain) {
        self.terrain.set(x, y, terrain);
    }

    pub fn is_locked_tile(&self, x: usize, y: usize) -> bool {
        *self.locked.get(x, y).unwrap_or(&false)
    }

    pub fn locked_grid(&self) -> &Grid<bool> {
        &self.locked
    }

    pub fn get_all_grids_mut(
        &mut self,
    ) -> (&mut Grid<Terrain>, &mut Grid<Vec<Prefab>>, &mut Grid<bool>) {
        (&mut self.terrain, &mut self.entities, &mut self.locked)
    }

    pub fn get_terrain_and_locked_mut(&mut self) -> (&mut Grid<Terrain>, &mut Grid<bool>) {
        (&mut self.terrain, &mut self.locked)
    }
}
