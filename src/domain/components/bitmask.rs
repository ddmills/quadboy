use bevy_ecs::prelude::*;

use crate::{
    cfg::CARDINALS_OFFSET,
    domain::Zone,
    rendering::{Glyph, Position, RecordZonePosition},
};

#[derive(Clone, Copy)]
pub enum BitmaskStyle {
    Wall,
}

#[derive(Resource)]
pub struct Bitmasker {
    glyphs: Vec<usize>,
}

impl Default for Bitmasker {
    fn default() -> Self {
        Self {
            glyphs: vec![240, 241],
        }
    }
}

impl Bitmasker {
    pub fn sum_mask<F>(fun: F) -> usize
    where
        F: Fn(i32, i32) -> bool,
    {
        CARDINALS_OFFSET
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

    pub fn get_mask_idx(_: BitmaskStyle, mask: usize) -> usize {
        // 2nd bit is tile below
        let below = !(mask >> 2).is_multiple_of(2);

        if below { 0 } else { 1 }
    }

    pub fn get_glyph_idx(&self, mask_idx: usize) -> usize {
        *self.glyphs.get(mask_idx).unwrap_or(&0)
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

        let sum = Bitmasker::sum_mask(|ox, oy| {
            let dx = x as i32 + ox;
            let dy = y as i32 + oy;

            if dx < 0 || dy < 0 {
                return false;
            }

            let list = Zone::get_at((dx as usize, dy as usize, z), &q_zones);

            for e in list.iter() {
                if q_bitmasks.contains(*e) {
                    return true;
                }
            }

            false
        });

        let mask_idx = Bitmasker::get_mask_idx(bitmask.style, sum);
        let glyph_idx = bitmasker.get_glyph_idx(mask_idx);

        let mut glyph = q_glyphs.get_mut(*entity).unwrap();
        glyph.idx = glyph_idx;
    }
}
