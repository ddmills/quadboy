use bevy_ecs::prelude::*;
use std::collections::HashSet;

use crate::{
    DebugMode,
    common::Palette,
    domain::{AiController, AiState, Player, PlayerPosition, PursuingTarget, VisionBlocker, Zone},
    rendering::{Layer, Position, Text, Visibility, world_to_zone_idx, world_to_zone_local},
    states::CleanupStateExplore,
};

#[derive(Component)]
pub struct AiDebugIndicator {
    pub ai_entity: Entity,
}

pub fn render_ai_debug_indicators(
    debug_mode: Res<DebugMode>,
    player_pos: Res<PlayerPosition>,
    q_zones: Query<&Zone>,
    mut queries: ParamSet<(
        Query<(Entity, &Position, &AiController), (Without<Player>, Without<AiDebugIndicator>)>,
        Query<(&AiDebugIndicator, &mut Position, &mut Text)>,
        Query<(&Position, &AiController), (Without<Player>, With<AiController>)>,
        Query<&Position, With<VisionBlocker>>,
        Query<&Position, With<Player>>,
    )>,
    q_pursuing: Query<&PursuingTarget>,
    q_existing_indicators: Query<Entity, With<AiDebugIndicator>>,
    mut cmds: Commands,
) {
    if !debug_mode.ai_debug {
        // Only clear indicators when debug mode is OFF
        for entity in q_existing_indicators.iter() {
            cmds.entity(entity).despawn();
        }
        return;
    }

    // Get visible zone bounds
    let player_zone = player_pos.zone_idx();
    let zone = q_zones.iter().find(|z| z.idx == player_zone);
    if zone.is_none() {
        return;
    }

    // Collect vision blockers and player position first to avoid ParamSet conflicts
    let vision_blockers: Vec<Position> = queries.p3().iter().cloned().collect();
    let player_position = queries.p4().single().ok().cloned();

    // Update existing indicators and create new ones as needed

    // First, collect indicator entity IDs and their AI entity references
    let indicator_mappings: Vec<(Entity, Entity)> = queries
        .p1()
        .iter()
        .map(|(indicator, _, _)| (indicator.ai_entity, indicator.ai_entity))
        .collect();

    // Then collect the AI data we need for updates
    let mut ai_data = Vec::new();
    for (ai_entity, _) in &indicator_mappings {
        if let Ok((ai_position, ai_controller)) = queries.p2().get(*ai_entity) {
            // Only show indicators for entities in the visible zone
            if ai_position.zone_idx() != player_zone {
                continue;
            }

            let (state_text, state_color) = get_ai_state_display_simple(
                &ai_controller.state,
                ai_position,
                ai_controller,
                &vision_blockers,
                &player_position,
            );

            ai_data.push((
                *ai_entity,
                ai_position.x,
                ai_position.y - 0.5,
                ai_position.z,
                state_text,
                state_color,
            ));
        }
    }

    // Now apply the updates to indicators
    for (indicator, mut indicator_pos, mut indicator_text) in queries.p1().iter_mut() {
        if let Some((_, x, y, z, text, color)) = ai_data
            .iter()
            .find(|(entity, _, _, _, _, _)| *entity == indicator.ai_entity)
        {
            indicator_pos.x = *x;
            indicator_pos.y = *y;
            indicator_pos.z = *z;
            indicator_text.value = text.clone();
            indicator_text.fg1 = Some(*color as u32);
        }
    }

    // Then, create indicators for AI entities that don't have them yet
    let existing_ai_entities: HashSet<Entity> = queries
        .p1()
        .iter()
        .map(|(indicator, _, _)| indicator.ai_entity)
        .collect();

    for (entity, position, ai_controller) in queries.p0().iter() {
        // Only show indicators for entities in the visible zone
        if position.zone_idx() != player_zone {
            continue;
        }

        // Skip if this AI entity already has an indicator
        if existing_ai_entities.contains(&entity) {
            continue;
        }

        let (state_text, state_color) = get_ai_state_display_simple(
            &ai_controller.state,
            position,
            ai_controller,
            &vision_blockers,
            &player_position,
        );
        let is_pursuing = q_pursuing.get(entity).is_ok();

        let indicator_pos = Position::new_f32(position.x, position.y - 0.5, position.z);

        let text_component = Text::new(&state_text)
            .fg1(state_color)
            .outline(Palette::Black)
            .layer(Layer::Overlay);

        cmds.spawn((
            AiDebugIndicator { ai_entity: entity },
            text_component,
            indicator_pos,
            Visibility::Visible,
            CleanupStateExplore,
        ));
    }
}

fn has_line_of_sight_simple(
    from_pos: &Position,
    to_pos: &Position,
    vision_blockers: &[Position],
) -> bool {
    let (from_x, from_y, from_z) = from_pos.world();
    let (to_x, to_y, to_z) = to_pos.world();

    // Must be on same Z level for line of sight
    if from_z != to_z {
        return false;
    }

    // Use Bresenham's line algorithm to check for blockers along the path
    let dx = (to_x as i32 - from_x as i32).abs();
    let dy = (to_y as i32 - from_y as i32).abs();
    let mut x = from_x as i32;
    let mut y = from_y as i32;
    let x_inc = if from_x < to_x { 1 } else { -1 };
    let y_inc = if from_y < to_y { 1 } else { -1 };
    let mut error = dx - dy;

    loop {
        // Check if there's a vision blocker at current position
        for blocker_pos in vision_blockers {
            let (blocker_x, blocker_y, blocker_z) = blocker_pos.world();
            if blocker_x == x as usize && blocker_y == y as usize && blocker_z == from_z {
                // Don't count blockers at the start or end positions
                if (x as usize, y as usize) != (from_x, from_y)
                    && (x as usize, y as usize) != (to_x, to_y)
                {
                    return false;
                }
            }
        }

        // Reached target
        if x == to_x as i32 && y == to_y as i32 {
            break;
        }

        let e2 = 2 * error;
        if e2 > -dy {
            error -= dy;
            x += x_inc;
        }
        if e2 < dx {
            error += dx;
            y += y_inc;
        }
    }

    true
}

fn get_ai_state_display_simple(
    state: &AiState,
    ai_pos: &Position,
    ai_controller: &AiController,
    vision_blockers: &[Position],
    player_position: &Option<Position>,
) -> (String, Palette) {
    let mut visibility_indicator = "";

    // Check if AI can see its target (player for now)
    if let Some(player_pos) = player_position {
        // For pursuing state, check if they can see the player
        if matches!(state, AiState::Pursuing) || ai_controller.current_target_id.is_some() {
            if has_line_of_sight_simple(ai_pos, player_pos, vision_blockers) {
                visibility_indicator = "(v)";
            }
        }
    }

    let base_text = match state {
        AiState::Idle => "idle",
        AiState::Wandering => "wander",
        AiState::Pursuing => "pursue",
        AiState::Fighting => "fight",
        AiState::Fleeing => "flee",
        AiState::Returning => "return",
        AiState::Waiting => "wait",
    };

    let color = match state {
        AiState::Idle => Palette::Green,
        AiState::Wandering => Palette::Blue,
        AiState::Pursuing => Palette::Red,
        AiState::Fighting => Palette::DarkRed,
        AiState::Fleeing => Palette::Yellow,
        AiState::Returning => Palette::Orange,
        AiState::Waiting => Palette::Purple,
    };

    (format!("{}{}", base_text, visibility_indicator), color)
}
