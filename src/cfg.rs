pub const TILE_SIZE: (usize, usize) = (16, 24);
pub const TILE_SIZE_F32: (f32, f32) = (TILE_SIZE.0 as f32, TILE_SIZE.1 as f32);

pub const BODY_FONT_SIZE: (usize, usize) = (8, 12);
pub const BODY_FONT_SIZE_F32: (f32, f32) = (BODY_FONT_SIZE.0 as f32, BODY_FONT_SIZE.1 as f32);

pub const TEXEL_SIZE: u32 = 2;
pub const TEXEL_SIZE_F32: f32 = TEXEL_SIZE as f32;

pub const MAP_SIZE: (usize, usize, usize) = (40, 20, 1);
pub const ZONE_SIZE: (usize, usize) = (80, 60);
pub const ZONE_SIZE_F32: (f32, f32) = (ZONE_SIZE.0 as f32, ZONE_SIZE.1 as f32);

pub const WINDOW_SIZE: (usize, usize) = (928, 720);

pub const INPUT_RATE: f64 = 0.025;
pub const INPUT_INITIAL_DELAY: f64 = 0.25;

pub const ENABLE_SAVES: bool = true;
