use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;

use crate::{
    common::{
        MacroquadColorable,
        algorithm::shadowcast::{ShadowcastSettings, shadowcast},
    },
    domain::{
        ColliderFlags, EquipmentSlots, Equipped, InActiveZone, LightSource, Overworld,
        PlayerPosition, Zone, Zones,
    },
    engine::{Clock, StableId, StableIdRegistry},
    rendering::{LightingData, Position, world_to_zone_local},
};

#[derive(Clone, Copy)]
struct LightContribution {
    r: f32,
    g: f32,
    b: f32,
    intensity: f32,
    flicker: f32,
    source_id: Entity,
}

#[profiled_system]
pub fn update_lighting_system(
    clock: Res<Clock>,
    player_pos: Res<PlayerPosition>,
    overworld: Res<Overworld>,
    zones: Res<Zones>,
    mut lighting_data: ResMut<LightingData>,
    registry: Res<StableIdRegistry>,
    q_lights: Query<(Entity, &Position, &LightSource), With<InActiveZone>>,
    q_zones: Query<&Zone>,
    q_equipped_lights: Query<&LightSource, With<Equipped>>,
    q_entities_with_equipment: Query<(&Position, &EquipmentSlots), With<InActiveZone>>,
) {
    if clock.tick_delta_accum() == 0 {
        return;
    }

    let player_zone_idx = player_pos.zone_idx();
    let biome_type = overworld.get_zone_type(player_zone_idx);

    let (ambient_color, ambient_intensity) = {
        // Calculate ambient light based on biome and daylight
        if biome_type.uses_daylight_cycle() {
            let daylight = clock.get_daylight();
            let biome_color_rgba = biome_type.get_ambient_color().to_rgba(1.0);
            let biome_intensity = biome_type.get_ambient_intensity();

            // Blend biome color with daylight, preserving more daylight color at night
            let night_blend_factor = (1.0 - daylight.intensity).powf(1.2) * 0.3;
            let blended_r = daylight.color.x * biome_color_rgba[0] * (1.0 - night_blend_factor)
                + daylight.color.x * night_blend_factor;
            let blended_g = daylight.color.y * biome_color_rgba[1] * (1.0 - night_blend_factor)
                + daylight.color.y * night_blend_factor;
            let blended_b = daylight.color.z * biome_color_rgba[2] * (1.0 - night_blend_factor)
                + daylight.color.z * night_blend_factor;

            let final_r = (blended_r * 255.0) as u32;
            let final_g = (blended_g * 255.0) as u32;
            let final_b = (blended_b * 255.0) as u32;
            let final_color = (final_r << 16) | (final_g << 8) | final_b;
            let final_intensity = daylight.intensity * biome_intensity;

            (final_color, final_intensity)
        } else {
            (
                biome_type.get_ambient_color(),
                biome_type.get_ambient_intensity(),
            )
        }
    };

    lighting_data.clear();
    lighting_data.set_ambient(ambient_color, ambient_intensity);

    let Some(zone_entity) = zones.cache.get(&player_zone_idx) else {
        return;
    };

    let Ok(zone) = q_zones.get(*zone_entity) else {
        return;
    };

    let all_fragments: HashMap<(i32, i32), Vec<LightContribution>> = {
        let mut all_fragments: HashMap<(i32, i32), Vec<LightContribution>> = HashMap::new();

        for (entity, pos, light) in q_lights
            .iter()
            .filter(|(_, pos, light)| light.is_enabled && pos.zone_idx() == player_zone_idx)
        {
            apply_light_source(pos, light, entity, zone, &mut all_fragments);
        }

        for (owner_pos, equipment_slots) in q_entities_with_equipment
            .iter()
            .filter(|(pos, _)| pos.zone_idx() == player_zone_idx)
        {
            for slot_id in equipment_slots.slots.values().filter_map(|&id| id) {
                if let Some(item_entity) = registry.get_entity(StableId(slot_id))
                    && let Ok(light) = q_equipped_lights.get(item_entity)
                    && light.is_enabled
                {
                    apply_light_source(
                        owner_pos,
                        light,
                        item_entity,
                        zone,
                        &mut all_fragments,
                    );
                }
            }
        }

        all_fragments
    };

    let (floor_fragments, wall_fragments) = {
        let mut floor_fragments: HashMap<(i32, i32), Vec<LightContribution>> = HashMap::new();
        let mut wall_fragments: HashMap<(i32, i32), Vec<LightContribution>> = HashMap::new();

        for ((x, y), contributions) in all_fragments {
            let is_wall = zone
                .colliders
                .get_flags(x as usize, y as usize)
                .contains(ColliderFlags::BLOCKS_SIGHT);

            if is_wall {
                wall_fragments.insert((x, y), contributions);
            } else {
                apply_combined_light(x, y, &contributions, &mut lighting_data);
                floor_fragments.insert((x, y), contributions);
            }
        }

        (floor_fragments, wall_fragments)
    };

    {
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
    }
}

fn apply_light_source(
    pos: &Position,
    light: &LightSource,
    source_entity: Entity,
    zone: &Zone,
    all_fragments: &mut HashMap<(i32, i32), Vec<LightContribution>>,
) {
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
        is_blocker: |x, y| {
            if x < 0 || y < 0 {
                return true;
            }
            zone.colliders
                .get_flags(x as usize, y as usize)
                .contains(ColliderFlags::BLOCKS_SIGHT)
        },
        on_light: |x, y, distance| {
            if x < 0 || y < 0 || light_z < 0 {
                return;
            }

            let d = distance as f32;

            let intensity = if d == 0.0 {
                light.intensity
            } else {
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

            all_fragments.entry((x, y)).or_default().push(contribution);
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
    let dx = wall_x - player_x;
    let dy = wall_y - player_y;

    let (offset_x, offset_y) = match (dx.signum(), dy.signum()) {
        (1, 0) => (-1, 0),
        (-1, 0) => (1, 0),
        (0, 1) => (0, -1),
        (0, -1) => (0, 1),
        (1, 1) => (-1, -1),
        (1, -1) => (-1, 1),
        (-1, 1) => (1, -1),
        (-1, -1) => (1, 1),
        _ => (0, 0),
    };

    let adjacent_floor_pos = (wall_x + offset_x, wall_y + offset_y);

    if let Some(floor_lights) = floor_contributions.get(&adjacent_floor_pos) {
        let floor_source_ids: HashSet<Entity> = floor_lights
            .iter()
            .map(|contrib| contrib.source_id)
            .collect();

        let valid_contributions: Vec<_> = wall_contributions
            .iter()
            .filter(|wall_contrib| floor_source_ids.contains(&wall_contrib.source_id))
            .copied()
            .collect();

        if !valid_contributions.is_empty() {
            apply_combined_light(wall_x, wall_y, &valid_contributions, lighting_data);
        }
    }
}
