use crate::cfg::{MAP_SIZE, ZONE_SIZE};


// Convert a zone position to a zone index
#[inline]
pub fn zone_idx(x: usize, y: usize, z: usize) -> usize {
    x * MAP_SIZE.1 * MAP_SIZE.2 + y * MAP_SIZE.2 + z
}

// Convert a zone index to a zone position
#[inline]
pub fn zone_xyz(zone_idx: usize) -> (usize, usize, usize) {
    (
        zone_idx / (MAP_SIZE.1 * MAP_SIZE.2),
        (zone_idx / MAP_SIZE.2) % MAP_SIZE.1,
        zone_idx % MAP_SIZE.2,
    )
}

// convert local zone coordinates to world coordinates
#[inline]
pub fn zone_local_to_world(zone_idx: usize, x: usize, y: usize) -> (usize, usize, usize) {
    let cpos: (usize, usize, usize) = zone_xyz(zone_idx);

    (cpos.0 * ZONE_SIZE.0 + x, cpos.1 * ZONE_SIZE.1 + y, cpos.2)
}
