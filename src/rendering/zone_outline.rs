use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    cfg::ZONE_SIZE,
    common::Palette,
    domain::{PlayerPosition, Zones},
    engine::SerializableComponent,
    rendering::{
        Glyph, GlyphTextureId, Layer, Position, Visibility, world_to_zone_idx, zone_local_to_world,
    },
    states::CleanupStateExplore,
};

#[derive(Component, Clone, Serialize, Deserialize, SerializableComponent)]
pub struct ZoneOutlineTile;

#[derive(Resource)]
pub struct ZoneOutlineState {
    pub current_zone: Option<usize>,
}

impl Default for ZoneOutlineState {
    fn default() -> Self {
        Self { current_zone: None }
    }
}

pub fn spawn_zone_outline(
    mut cmds: Commands,
    player_pos: Res<PlayerPosition>,
    mut outline_state: ResMut<ZoneOutlineState>,
    q_existing: Query<Entity, With<ZoneOutlineTile>>,
) {
    // let player_zone_idx = world_to_zone_idx(
    //     player_pos.x as usize,
    //     player_pos.y as usize,
    //     player_pos.z as usize,
    // );

    // if outline_state.current_zone == Some(player_zone_idx) {
    //     return;
    // }

    // // Despawn existing outline entities
    // for entity in q_existing.iter() {
    //     cmds.entity(entity).despawn();
    // }

    // // Update current zone
    // outline_state.current_zone = Some(player_zone_idx);

    // let zone_width = ZONE_SIZE.0;
    // let zone_height = ZONE_SIZE.1;

    // // Spawn border tiles
    // for x in 0..zone_width {
    //     for y in 0..zone_height {
    //         // Only render border positions
    //         let is_border = x == 0 || x == zone_width - 1 || y == 0 || y == zone_height - 1;
    //         if !is_border {
    //             continue;
    //         }

    //         // Determine glyph index based on position
    //         let glyph_idx = match (x, y) {
    //             // Corners
    //             (0, 0) => 2,                                                // top-left
    //             (x, 0) if x == zone_width - 1 => 3,                         // top-right
    //             (0, y) if y == zone_height - 1 => 6,                        // bottom-left
    //             (x, y) if x == zone_width - 1 && y == zone_height - 1 => 7, // bottom-right
    //             // Edges
    //             (_, 0) => 8,                         // top
    //             (_, y) if y == zone_height - 1 => 9, // bottom
    //             (x, _) if x == zone_width - 1 => 11, // right
    //             (0, _) => 12,                        // left
    //             _ => continue,
    //         };

    //         // Convert local zone coordinates to world coordinates
    //         let (world_x, world_y, world_z) = zone_local_to_world(player_zone_idx, x, y);

    //         cmds.spawn((
    //             Glyph {
    //                 idx: glyph_idx,
    //                 fg1: Some(Palette::Gray as u32),
    //                 fg2: None,
    //                 bg: None,
    //                 outline: None,
    //                 outline_override: None,
    //                 position_offset: None,
    //                 scale: (1.0, 1.0),
    //                 layer_id: Layer::GroundOverlay,
    //                 texture_id: GlyphTextureId::Bitmasks,
    //                 is_dormant: false,
    //                 alpha: 1.0,
    //             },
    //             Position::new(world_x, world_y, world_z),
    //             Visibility::Visible,
    //             ZoneOutlineTile,
    //             CleanupStateExplore,
    //         ));
    //     }
    // }
}

pub fn setup_zone_outline_state(mut cmds: Commands) {
    cmds.insert_resource(ZoneOutlineState::default());
}
