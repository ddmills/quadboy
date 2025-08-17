use crate::{
    cfg::ZONE_SIZE,
    common::{Grid},
    domain::{Terrain, ZoneSaveData},
};

pub fn gen_zone(zone_idx: usize) -> ZoneSaveData {
    // let mut rand = Rand::seed(zone_idx as u64);
    let terrains = [
        Terrain::Dirt,
        Terrain::Dirt,
        Terrain::Dirt,
        Terrain::Dirt,
        Terrain::Grass,
        Terrain::Grass,
        Terrain::Grass,
        Terrain::Grass,
        Terrain::River,
    ];

    // let terrain = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_x, _y| rand.pick(&terrains));
    let terrain = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_x, _y| Terrain::Grass);

    ZoneSaveData {
        idx: zone_idx,
        terrain,
        entities: vec![],
    }
}
