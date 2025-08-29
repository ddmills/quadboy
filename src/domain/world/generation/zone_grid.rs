use crate::{
    cfg::ZONE_SIZE,
    common::Grid,
    domain::{SpawnConfig, Terrain},
};

pub struct ZoneGridData {
    pub terrain: Grid<Terrain>,
    pub entities: Grid<Vec<SpawnConfig>>,
    pub locked: Grid<bool>,
}

impl ZoneGridData {
    pub fn new() -> Self {
        Self {
            terrain: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| Terrain::Grass),
            entities: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| vec![]),
            locked: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| false),
        }
    }

    pub fn push_entity(&mut self, x: usize, y: usize, config: SpawnConfig) {
        if let Some(ents) = self.entities.get_mut(x, y) {
            ents.push(config);
        }
    }

    pub fn lock_tile(&mut self, x: usize, y: usize) {
        self.locked.set(x, y, true);
    }

    pub fn set_terrain(&mut self, x: usize, y: usize, terrain: Terrain) {
        self.terrain.set(x, y, terrain);
    }

    pub fn is_locked_tile(&self, x: usize, y: usize) -> bool {
        *self.locked.get(x, y).unwrap_or(&false)
    }

    pub fn terrain_grid(&self) -> &Grid<Terrain> {
        &self.terrain
    }

    pub fn terrain_grid_mut(&mut self) -> &mut Grid<Terrain> {
        &mut self.terrain
    }

    pub fn entities_grid(&self) -> &Grid<Vec<SpawnConfig>> {
        &self.entities
    }

    pub fn entities_grid_mut(&mut self) -> &mut Grid<Vec<SpawnConfig>> {
        &mut self.entities
    }

    pub fn locked_grid(&self) -> &Grid<bool> {
        &self.locked
    }

    pub fn locked_grid_mut(&mut self) -> &mut Grid<bool> {
        &mut self.locked
    }

    pub fn fill_unlocked_terrain(&mut self, terrain: Terrain) {
        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if !self.is_locked_tile(x, y) {
                    self.set_terrain(x, y, terrain);
                }
            }
        }
    }

    pub fn get_all_grids_mut(
        &mut self,
    ) -> (
        &mut Grid<Terrain>,
        &mut Grid<Vec<SpawnConfig>>,
        &mut Grid<bool>,
    ) {
        (&mut self.terrain, &mut self.entities, &mut self.locked)
    }

    pub fn get_terrain_and_locked_mut(&mut self) -> (&mut Grid<Terrain>, &mut Grid<bool>) {
        (&mut self.terrain, &mut self.locked)
    }
}
