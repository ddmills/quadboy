use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::*;
use macroquad::telemetry;

use crate::{
    common::{
        MacroquadColorable,
        algorithm::shadowcast::{ShadowcastSettings, shadowcast},
    },
    domain::{InActiveZone, LightBlocker, LightSource, Overworld, PlayerPosition},
    engine::Clock,
    rendering::{LightingData, Position, world_to_zone_local},
};

#[derive(Clone)]
struct LightContribution {
    r: f32,
    g: f32,
    b: f32,
    intensity: f32,
    flicker: f32,
    source_id: Entity, // Track which light source created this contribution
}

pub fn update_lighting_system(
    clock: Res<Clock>,
    player_pos: Res<PlayerPosition>,
    overworld: Res<Overworld>,
    mut lighting_data: ResMut<LightingData>,
    q_lights: Query<(Entity, &Position, &LightSource), With<InActiveZone>>,
    q_blockers: Query<&Position, (With<LightBlocker>, With<InActiveZone>)>,
) {
    telemetry::begin_zone("update_lighting_system");

    if clock.is_frozen() {
        telemetry::end_zone();
        return;
    }

    let player_zone_idx = player_pos.zone_idx();
    let biome_type = overworld.get_zone_type(player_zone_idx);

    // Calculate ambient light based on biome and daylight
    let (ambient_color, ambient_intensity) = if biome_type.uses_daylight_cycle() {
        let daylight = clock.get_daylight();
        let biome_color_rgba = biome_type.get_ambient_color().to_rgba(1.0);
        let biome_intensity = biome_type.get_ambient_intensity();

        // Calculate final color by multiplying daylight with biome color
        let final_r = (daylight.color.x * biome_color_rgba[0] * 255.0) as u32;
        let final_g = (daylight.color.y * biome_color_rgba[1] * 255.0) as u32;
        let final_b = (daylight.color.z * biome_color_rgba[2] * 255.0) as u32;
        let final_color = (final_r << 16) | (final_g << 8) | final_b;
        let final_intensity = daylight.intensity * biome_intensity;

        (final_color, final_intensity)
    } else {
        (
            biome_type.get_ambient_color(),
            biome_type.get_ambient_intensity(),
        )
    };

    lighting_data.clear();
    lighting_data.set_ambient(ambient_color, ambient_intensity);

    // Build blocker set for fast lookups - only include blockers in player's zone
    let blocker_positions: HashSet<_> = q_blockers
        .iter()
        .filter(|pos| pos.zone_idx() == player_zone_idx)
        .map(|pos| {
            let (local_x, local_y) = pos.zone_local();
            (local_x as i32, local_y as i32)
        })
        .collect();

    // Phase 1: Collect all light fragments (single collection like Haxe)
    let mut all_fragments: HashMap<(i32, i32), Vec<LightContribution>> = HashMap::new();

    // Process all enabled lights in the zone
    for (entity, pos, light) in q_lights.iter() {
        if !light.is_enabled || pos.zone_idx() != player_zone_idx {
            continue;
        }

        apply_light_source(pos, light, entity, &blocker_positions, &mut all_fragments);
    }

    // Phase 2: Separate floors and walls, apply floors immediately
    let mut floor_fragments: HashMap<(i32, i32), Vec<LightContribution>> = HashMap::new();
    let mut wall_fragments: HashMap<(i32, i32), Vec<LightContribution>> = HashMap::new();

    for ((x, y), contributions) in all_fragments {
        if blocker_positions.contains(&(x, y)) {
            // This is a wall
            wall_fragments.insert((x, y), contributions);
        } else {
            // This is a floor - apply immediately and store for POV checking
            apply_combined_light(x, y, &contributions, &mut lighting_data);
            floor_fragments.insert((x, y), contributions);
        }
    }

    // Phase 3: Apply POV-based wall lighting
    let player_world_pos = player_pos.world();
    let (player_local_x, player_local_y) =
        world_to_zone_local(player_world_pos.0, player_world_pos.1);

    for ((wall_x, wall_y), wall_contributions) in wall_fragments {
        apply_pov_wall_lighting(
            wall_x,
            wall_y,
            player_local_x as i32,
            player_local_y as i32,
            &wall_contributions,
            &floor_fragments,
            &mut lighting_data,
        );
    }

    telemetry::end_zone();
}

fn apply_light_source(
    pos: &Position,
    light: &LightSource,
    source_entity: Entity,
    blocker_positions: &HashSet<(i32, i32)>,
    all_fragments: &mut HashMap<(i32, i32), Vec<LightContribution>>,
) {
    // Convert color once
    let r = ((light.color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((light.color >> 8) & 0xFF) as f32 / 255.0;
    let b = (light.color & 0xFF) as f32 / 255.0;

    let (local_x, local_y) = pos.zone_local();
    let light_x = local_x as i32;
    let light_y = local_y as i32;
    let light_z = pos.z as i32;

    let settings = ShadowcastSettings {
        start_x: light_x,
        start_y: light_y,
        distance: light.range,
        is_blocker: |x, y| blocker_positions.contains(&(x, y)),
        on_light: |x, y, distance| {
            // Skip out of bounds (coordinates are already zone-local)
            if x < 0 || y < 0 || light_z < 0 {
                return;
            }

            // Calculate light intensity with falloff
            let d = distance as f32;

            let intensity = if d == 0.0 {
                light.intensity
            } else {
                // Smoothstep falloff: smooth S-curve from 1.0 to 0.0
                let t = (d / light.range as f32).min(1.0);
                let smoothstep = 1.0 - (t * t * (3.0 - 2.0 * t));
                light.intensity * smoothstep
            };

            let contribution = LightContribution {
                r,
                g,
                b,
                intensity,
                flicker: light.flicker,
                source_id: source_entity,
            };

            // Store ALL fragments together (don't separate walls/floors yet)
            all_fragments
                .entry((x, y))
                .or_default()
                .push(contribution);
        },
    };

    shadowcast(settings);
}

fn apply_combined_light(
    x: i32,
    y: i32,
    contributions: &[LightContribution],
    lighting_data: &mut LightingData,
) {
    let mut total_r = 0.0;
    let mut total_g = 0.0;
    let mut total_b = 0.0;
    let mut total_intensity = 0.0;
    let mut weighted_flicker = 0.0;

    for contribution in contributions {
        let weight = contribution.intensity;
        total_r += contribution.r * weight;
        total_g += contribution.g * weight;
        total_b += contribution.b * weight;
        total_intensity += contribution.intensity;
        weighted_flicker += contribution.flicker * weight;
    }

    if total_intensity > 0.0 {
        // Normalize colors by total intensity
        total_r /= total_intensity;
        total_g /= total_intensity;
        total_b /= total_intensity;
        weighted_flicker /= total_intensity;

        lighting_data.blend_light(
            x,
            y,
            total_r,
            total_g,
            total_b,
            total_intensity.min(1.0),
            weighted_flicker,
        );
    }
}

fn apply_pov_wall_lighting(
    wall_x: i32,
    wall_y: i32,
    player_x: i32,
    player_y: i32,
    wall_contributions: &[LightContribution],
    floor_contributions: &HashMap<(i32, i32), Vec<LightContribution>>,
    lighting_data: &mut LightingData,
) {
    // Calculate direction from player to wall
    let dx = wall_x - player_x;
    let dy = wall_y - player_y;

    // Get direction offset including diagonals (8-directional)
    let (offset_x, offset_y) = match (dx.signum(), dy.signum()) {
        (1, 0) => (-1, 0),  // East -> West
        (-1, 0) => (1, 0),  // West -> East
        (0, 1) => (0, -1),  // South -> North
        (0, -1) => (0, 1),  // North -> South
        (1, 1) => (-1, -1), // Southeast -> Northwest
        (1, -1) => (-1, 1), // Northeast -> Southwest
        (-1, 1) => (1, -1), // Southwest -> Northeast
        (-1, -1) => (1, 1), // Northwest -> Southeast
        _ => (0, 0),        // Same position (shouldn't happen)
    };

    // Check the floor tile adjacent to the wall (in player's direction)
    let adjacent_floor_pos = (wall_x + offset_x, wall_y + offset_y);

    if let Some(floor_lights) = floor_contributions.get(&adjacent_floor_pos) {
        // Find matching light contributions by source ID (like Haxe)
        let valid_contributions: Vec<_> = wall_contributions
            .iter()
            .filter(|wall_contrib| {
                // Check if this light source also lights the adjacent floor
                floor_lights
                    .iter()
                    .any(|floor_contrib| wall_contrib.source_id == floor_contrib.source_id)
            })
            .cloned()
            .collect();

        if !valid_contributions.is_empty() {
            apply_combined_light(wall_x, wall_y, &valid_contributions, lighting_data);
        }
    }
}
