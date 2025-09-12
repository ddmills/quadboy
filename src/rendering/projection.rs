use crate::cfg::{MAP_SIZE, ZONE_SIZE, ZONE_SIZE_F32};

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

#[allow(dead_code)]
#[inline]
pub fn is_zone_oob(x: usize, y: usize, z: usize) -> bool {
    x >= MAP_SIZE.0 || y >= MAP_SIZE.1 || z >= MAP_SIZE.2
}

// convert local zone coordinates to world coordinates
#[inline]
pub fn zone_local_to_world(zone_idx: usize, x: usize, y: usize) -> (usize, usize, usize) {
    let zpos = zone_xyz(zone_idx);

    (zpos.0 * ZONE_SIZE.0 + x, zpos.1 * ZONE_SIZE.1 + y, zpos.2)
}

// convert local zone coordinates to world coordinates
#[inline]
pub fn zone_local_to_world_f32(zone_idx: usize, x: f32, y: f32) -> (f32, f32, f32) {
    let zpos = zone_xyz(zone_idx);

    (
        (zpos.0 as f32) * ZONE_SIZE_F32.0 + x,
        (zpos.1 as f32) * ZONE_SIZE_F32.1 + y,
        zpos.2 as f32,
    )
}

// convert world coordinates to local zone coordinates
#[inline]
pub fn world_to_zone_local(x: usize, y: usize) -> (usize, usize) {
    (x % ZONE_SIZE.0, y % ZONE_SIZE.1)
}

// convert world coordinates to local zone coordinates
#[inline]
pub fn world_to_zone_local_f32(x: f32, y: f32) -> (f32, f32) {
    (x % ZONE_SIZE_F32.0, y % ZONE_SIZE_F32.1)
}

#[inline]
pub fn world_to_zone_idx(x: usize, y: usize, z: usize) -> usize {
    let cpos = (x / ZONE_SIZE.0, y / ZONE_SIZE.1, z);

    zone_idx(cpos.0, cpos.1, cpos.2)
}

pub fn zone_center_world(zone_idx: usize) -> (f32, f32) {
    let zone_pos = zone_xyz(zone_idx);
    (
        (zone_pos.0 * ZONE_SIZE.0) as f32 + (ZONE_SIZE_F32.0 / 2.),
        (zone_pos.1 * ZONE_SIZE.1) as f32 + (ZONE_SIZE_F32.1 / 2.),
    )
}
