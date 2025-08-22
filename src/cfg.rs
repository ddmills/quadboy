pub const TILE_SIZE: (usize, usize) = (16, 24);
pub const TILE_SIZE_F32: (f32, f32) = (TILE_SIZE.0 as f32, TILE_SIZE.1 as f32);

pub const BODY_FONT_SIZE: (usize, usize) = (8, 12);
pub const BODY_FONT_SIZE_F32: (f32, f32) = (BODY_FONT_SIZE.0 as f32, BODY_FONT_SIZE.1 as f32);

pub const TEXEL_SIZE: u32 = 2;
pub const TEXEL_SIZE_F32: f32 = TEXEL_SIZE as f32;

pub const MAP_SIZE: (usize, usize, usize) = (12, 8, 20);
pub const ZONE_SIZE: (usize, usize) = (80, 40);
pub const ZONE_SIZE_F32: (f32, f32) = (ZONE_SIZE.0 as f32, ZONE_SIZE.1 as f32);

pub const WINDOW_SIZE: (usize, usize) = (TILE_SIZE.0 * 70 + 12, TILE_SIZE.1 * 30 + 12);

pub const CARDINALS_OFFSET: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
