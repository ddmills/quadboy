use std::collections::HashMap;

use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    cfg::CARDINALS_OFFSET,
    domain::Zone,
    rendering::{Glyph, Position, RecordZonePosition},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitmaskStyle {
    Rocks,
    Outline,
}

impl BitmaskStyle {
    pub fn get_calculator(&self) -> BitmaskCalculator {
        match self {
            BitmaskStyle::Rocks => BitmaskCalculator::Basic,
            BitmaskStyle::Outline => BitmaskCalculator::Simple { idx_offset: 0 },
        }
    }

    pub fn get_compatible_styles(&self) -> Vec<BitmaskStyle> {
        match self {
            BitmaskStyle::Rocks => vec![BitmaskStyle::Rocks],
            BitmaskStyle::Outline => vec![BitmaskStyle::Outline],
        }
    }
}

pub enum BitmaskCalculator {
    Basic,
    Simple { idx_offset: usize },
}

#[derive(Resource)]
pub struct Bitmasker {
    old: Vec<usize>,
    glyphs: HashMap<BitmaskStyle, Vec<usize>>,
}

impl Default for Bitmasker {
    fn default() -> Self {
        let mut glyphs = HashMap::new();

        glyphs.insert(BitmaskStyle::Rocks, vec![240, 241]);
        glyphs.insert(
            BitmaskStyle::Outline,
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        );

        Self {
            old: vec![240, 241],
            glyphs,
        }
    }
}

impl Bitmasker {
    pub fn sum_mask<F>(fun: F) -> usize
    where
        F: Fn(i32, i32) -> bool,
    {
        // CARDINALS_OFFSET
        [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ]
        .iter()
        .enumerate()
        .fold(0_usize, |sum, (idx, (x, y))| {
            if fun(*x, *y) {
                sum + 2_usize.pow(idx as u32)
            } else {
                sum
            }
        })
    }

    pub fn get_mask_idx(calc: BitmaskCalculator, mask: usize) -> usize {
        match calc {
            BitmaskCalculator::Basic => {
                // 2nd bit is tile below
                let below = !(mask >> 2).is_multiple_of(2);

                if below { 0 } else { 1 }
            }
            BitmaskCalculator::Simple { idx_offset: _ } => {
                let mut x = mask;
                x &= !(1 << 0);
                x &= !(1 << 2);
                x &= !(1 << 5);
                x &= !(1 << 7);

                match x {
                    66 => 0,
                    0 => 1,
                    80 => 2,
                    72 => 3,
                    2 => 4,
                    24 => 5,
                    18 => 6,
                    10 => 7,
                    88 => 8,
                    26 => 9,
                    90 => 10,
                    74 => 11,
                    82 => 12,
                    16 => 13,
                    8 => 14,
                    64 => 15,
                    _ => 1,
                }
            }
        }
    }

    pub fn get_glyph_idx(&self, style: BitmaskStyle, mask_idx: usize) -> usize {
        let Some(glyphs) = self.glyphs.get(&style) else {
            return 0;
        };

        let calc = style.get_calculator();
        let Some(glyph_idx) = glyphs.get(mask_idx) else {
            return 0;
        };

        match calc {
            BitmaskCalculator::Basic => *glyph_idx,
            BitmaskCalculator::Simple { idx_offset } => glyph_idx + idx_offset,
        }
    }
}

#[derive(Component)]
pub struct BitmaskGlyph {
    pub style: BitmaskStyle,
}

impl BitmaskGlyph {
    pub fn new(style: BitmaskStyle) -> Self {
        Self { style }
    }
}

pub fn on_bitmask_spawn(
    q_bitmasks_new: Query<(Entity, &Position), (Added<BitmaskGlyph>, With<RecordZonePosition>)>,
    q_zones: Query<&Zone>,
    mut e_refresh_bitmask: EventWriter<RefreshBitmask>,
) {
    for (e, p) in q_bitmasks_new.iter() {
        e_refresh_bitmask.write(RefreshBitmask(e));

        let neighbors = Zone::get_neighbors(p.world(), &q_zones);

        for neighbor in neighbors.iter().flatten() {
            e_refresh_bitmask.write(RefreshBitmask(*neighbor));
        }
    }
}

#[derive(Event)]
pub struct RefreshBitmask(pub Entity);

pub fn on_refresh_bitmask(
    mut ev_refresh_bitmask: EventReader<RefreshBitmask>,
    q_bitmasks: Query<(&BitmaskGlyph, &Position)>,
    q_zones: Query<&Zone>,
    mut q_glyphs: Query<&mut Glyph>,
    bitmasker: Res<Bitmasker>,
) {
    for RefreshBitmask(entity) in ev_refresh_bitmask.read() {
        let Ok((bitmask, position)) = q_bitmasks.get(*entity) else {
            continue;
        };

        let (x, y, z) = position.world();

        let compatibles = bitmask.style.get_compatible_styles();

        let sum = Bitmasker::sum_mask(|ox, oy| {
            let dx = x as i32 + ox;
            let dy = y as i32 + oy;

            if dx < 0 || dy < 0 {
                return false;
            }

            let list = Zone::get_at((dx as usize, dy as usize, z), &q_zones);

            for e in list.iter() {
                let Ok((bm, _)) = q_bitmasks.get(*e) else {
                    continue;
                };

                if compatibles.contains(&bm.style) {
                    return true;
                }
            }

            false
        });

        let mask_idx = Bitmasker::get_mask_idx(bitmask.style.get_calculator(), sum);
        let glyph_idx = bitmasker.get_glyph_idx(bitmask.style, mask_idx);

        let mut glyph = q_glyphs.get_mut(*entity).unwrap();
        glyph.idx = glyph_idx;
    }
}
