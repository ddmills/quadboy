use bevy_ecs::prelude::*;

use crate::{
    common::algorithm::distance::Distance,
    domain::{AiController, FactionMember, Zone, get_effective_relationship},
    rendering::{Position, world_to_zone_idx, world_to_zone_local},
};

pub struct DetectedActors {
    pub entity: Entity,
    pub pos: (usize, usize, usize),
    pub distance: f32,
    pub relationship: i8,
}

pub fn detect_actors(world: &mut World, entity: Entity) -> Vec<DetectedActors> {
    // Get the AI entity's faction and position
    let (_our_faction_id, position_world, detection_range) = {
        let Some(ai_controller) = world.get::<AiController>(entity) else {
            return vec![];
        };

        let Some(position) = world.get::<Position>(entity) else {
            return vec![];
        };

        let Some(faction_member) = world.get::<FactionMember>(entity) else {
            return vec![];
        };

        (
            faction_member.faction_id,
            position.world(),
            ai_controller.detection_range,
        )
    };

    let mut enemies = Vec::new();

    // Get the zone index for our position
    let our_zone_idx = world_to_zone_idx(position_world.0, position_world.1, position_world.2);

    // Find the zone entity
    let zone_entity = {
        let mut zone_query = world.query::<(Entity, &Zone)>();
        zone_query
            .iter(world)
            .find(|(_, zone)| zone.idx == our_zone_idx)
            .map(|(zone_entity, _)| zone_entity)
    };

    let Some(zone_entity) = zone_entity else {
        return enemies;
    };

    let Some(zone) = world.get::<Zone>(zone_entity) else {
        return enemies;
    };

    // Search in a square area around our position within detection range
    let (center_x, center_y, center_z) = position_world;
    let range = detection_range as i32;

    for dx in -range..=range {
        for dy in -range..=range {
            // Calculate the world position to check
            let check_x = center_x as i32 + dx;
            let check_y = center_y as i32 + dy;

            // Skip if out of bounds
            if check_x < 0 || check_y < 0 {
                continue;
            }

            let check_pos = (check_x as usize, check_y as usize, center_z);

            // Check if this position is in the same zone
            let check_zone_idx = world_to_zone_idx(check_pos.0, check_pos.1, check_pos.2);
            if check_zone_idx != our_zone_idx {
                continue;
            }

            // Get local coordinates for this zone
            let (local_x, local_y) = world_to_zone_local(check_pos.0, check_pos.1);

            // Get entities at this position from the zone's entity grid
            if let Some(entities_at_pos) = zone.entities.get(local_x, local_y) {
                for &candidate_entity in entities_at_pos {
                    // Skip ourselves
                    if candidate_entity == entity {
                        continue;
                    }

                    let relationship = get_effective_relationship(entity, candidate_entity, world);

                    // Calculate distance using diagonal distance formula
                    let distance = Distance::diagonal(
                        [
                            position_world.0 as i32,
                            position_world.1 as i32,
                            position_world.2 as i32,
                        ],
                        [check_pos.0 as i32, check_pos.1 as i32, check_pos.2 as i32],
                    );

                    enemies.push(DetectedActors {
                        entity: candidate_entity,
                        pos: check_pos,
                        distance,
                        relationship,
                    });
                }
            }
        }
    }

    enemies
}
