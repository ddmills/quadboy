use bevy_ecs::prelude::*;
use std::collections::HashSet;

use crate::{
    DebugMode,
    common::Palette,
    domain::{AiController, AiState, Player, PlayerPosition, PursuingPlayer, Zone},
    rendering::{Layer, Position, Text, Visibility},
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
    )>,
    q_pursuing: Query<&PursuingPlayer>,
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

            let (state_text, state_color) = get_ai_state_display(&ai_controller.state);

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

        let (state_text, state_color) = get_ai_state_display(&ai_controller.state);
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

fn get_ai_state_display(state: &AiState) -> (String, Palette) {
    match state {
        AiState::Idle => ("idle".to_string(), Palette::Green),
        AiState::Wandering => ("wander".to_string(), Palette::Blue),
        AiState::Pursuing => ("pursue".to_string(), Palette::Red),
        AiState::Fighting => ("fight".to_string(), Palette::DarkRed),
        AiState::Fleeing => ("flee".to_string(), Palette::Yellow),
        AiState::Returning => ("return".to_string(), Palette::Orange),
        AiState::Waiting => ("wait".to_string(), Palette::Purple),
    }
}
