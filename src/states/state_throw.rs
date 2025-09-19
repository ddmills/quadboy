use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::{
    input::{KeyCode, MouseButton, is_key_pressed, is_mouse_button_pressed},
    prelude::trace,
};

use crate::{
    cfg::ZONE_SIZE,
    common::Palette,
    domain::{
        Attributes, BitmaskGlyph, BitmaskStyle, IgnoreLighting, Player, PlayerPosition,
        RefreshBitmask, ThrowItemAction, Throwable, Zone, game_loop,
    },
    engine::{App, Mouse, Plugin, StableIdRegistry},
    rendering::{
        Glyph, GlyphTextureId, Layer, Position, RecordZonePosition, Text, Visibility,
        world_to_zone_idx, world_to_zone_local,
    },
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
};

#[derive(Resource, Clone)]
pub struct ThrowContext {
    pub player_entity: Entity,
    pub item_id: u64,
    pub throw_range: usize,
}

#[derive(Component, Clone)]
pub struct CleanupStateThrow;

#[derive(Component)]
pub struct ThrowRangeIndicator;

#[derive(Component)]
pub struct ThrowCursor;

#[derive(Resource)]
struct ThrowCallbacks {
    back_to_inventory: SystemId,
}

fn back_to_inventory(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Inventory;
}

fn setup_throw_callbacks(world: &mut World) {
    let callbacks = ThrowCallbacks {
        back_to_inventory: world.register_system(back_to_inventory),
    };
    world.insert_resource(callbacks);
}

fn setup_throw_state(
    mut cmds: Commands,
    mut game_state: ResMut<CurrentGameState>,
    q_player: Query<(Entity, &Attributes), With<Player>>,
    q_throwable: Query<&Throwable>,
    registry: Res<StableIdRegistry>,
    context: Option<Res<ThrowContext>>,
    player_pos: Res<PlayerPosition>,
    q_zones: Query<&Zone>,
    mut e_refresh_bitmask: EventWriter<RefreshBitmask>,
) {
    let Some(context) = context else {
        eprintln!("ThrowContext not found - returning to inventory");
        game_state.next = GameState::Inventory;
        return;
    };

    let Ok((player_entity, attributes)) = q_player.single() else {
        return;
    };

    let Some(item_entity) = registry.get_entity(context.item_id) else {
        eprintln!("Item entity not found - returning to inventory");
        game_state.next = GameState::Inventory;
        return;
    };

    let Ok(throwable) = q_throwable.get(item_entity) else {
        eprintln!("Item is not throwable - returning to inventory");
        game_state.next = GameState::Inventory;
        return;
    };

    // Calculate throw range: base range + strength
    let throw_range = throwable.calculate_throw_range(attributes.strength);

    // Create updated context with calculated range
    let updated_context = ThrowContext {
        player_entity,
        item_id: context.item_id,
        throw_range,
    };

    // Update context resource
    cmds.insert_resource(updated_context.clone());

    // Spawn UI elements
    cmds.spawn((
        Text::new(&format!("THROW ITEM - Range: {}", throw_range))
            .fg1(Palette::Yellow)
            .layer(Layer::Ui),
        Position::new_f32(2., 1., 0.),
        CleanupStateThrow,
    ));

    cmds.spawn((
        Text::new("Click to throw | ESC to cancel")
            .fg1(Palette::Gray)
            .layer(Layer::Ui),
        Position::new_f32(2., 2., 0.),
        CleanupStateThrow,
    ));

    // Spawn throw cursor (initially hidden)
    cmds.spawn((
        Glyph::new(88, Palette::Red, Palette::Black).layer(Layer::Overlay),
        Position::new_f32(0., 0., 0.),
        Visibility::Hidden,
        ThrowCursor,
        CleanupStateThrow,
    ));

    // Spawn throw range indicators
    spawn_throw_range_indicators_impl(
        &mut cmds,
        &updated_context,
        &player_pos,
        &q_zones,
        &mut e_refresh_bitmask,
    );
}

fn spawn_throw_range_indicators_impl(
    cmds: &mut Commands,
    context: &ThrowContext,
    player_pos: &PlayerPosition,
    q_zones: &Query<&Zone>,
    e_refresh_bitmask: &mut EventWriter<RefreshBitmask>,
) {
    let player_world = player_pos.world();
    let player_zone_idx = world_to_zone_idx(player_world.0, player_world.1, player_world.2);

    // Find player's zone
    let Some(_zone) = q_zones.iter().find(|z| z.idx == player_zone_idx) else {
        return;
    };

    let (player_local_x, player_local_y) = world_to_zone_local(player_world.0, player_world.1);

    // Create range indicators in a circle around the player
    let range = context.throw_range as i32;
    for dx in -range..=range {
        for dy in -range..=range {
            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            if distance <= context.throw_range as f32 && distance > 0.0 {
                let target_local_x = player_local_x as i32 + dx;
                let target_local_y = player_local_y as i32 + dy;

                // Check bounds
                if target_local_x >= 0
                    && target_local_x < ZONE_SIZE.0 as i32
                    && target_local_y >= 0
                    && target_local_y < ZONE_SIZE.1 as i32
                {
                    let target_world_x = player_world.0 as i32 + dx;
                    let target_world_y = player_world.1 as i32 + dy;
                    let entity = cmds
                        .spawn((
                            Glyph::idx(0)
                                .fg1(Palette::White)
                                .fg2(Palette::White)
                                .texture(GlyphTextureId::Bitmasks)
                                .layer(Layer::Overlay),
                            BitmaskGlyph::new(BitmaskStyle::Outline),
                            RecordZonePosition,
                            IgnoreLighting,
                            Position::new_world((
                                target_world_x as usize,
                                target_world_y as usize,
                                player_world.2,
                            )),
                            Visibility::Visible,
                            ThrowRangeIndicator,
                            CleanupStateThrow,
                        ))
                        .id();

                    // Send refresh event for this entity
                    e_refresh_bitmask.write(RefreshBitmask(entity));
                }
            }
        }
    }

    // Refresh neighboring bitmask entities to ensure proper outline connections
    let throw_range = context.throw_range;
    for dx in -range..=range {
        for dy in -range..=range {
            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            if distance <= throw_range as f32 && distance > 0.0 {
                let target_local_x = player_local_x as i32 + dx;
                let target_local_y = player_local_y as i32 + dy;

                if target_local_x >= 0
                    && target_local_x < ZONE_SIZE.0 as i32
                    && target_local_y >= 0
                    && target_local_y < ZONE_SIZE.1 as i32
                {
                    let target_world_x = player_world.0 as i32 + dx;
                    let target_world_y = player_world.1 as i32 + dy;
                    let position = (
                        target_world_x as usize,
                        target_world_y as usize,
                        player_world.2,
                    );

                    let neighbors = Zone::get_neighbors(position, &q_zones);
                    for neighbor in neighbors.iter().flatten() {
                        e_refresh_bitmask.write(RefreshBitmask(*neighbor));
                    }
                }
            }
        }
    }
}

fn handle_throw_input(
    mut cmds: Commands,
    mut game_state: ResMut<CurrentGameState>,
    context: Res<ThrowContext>,
    mouse: Res<Mouse>,
    player_pos: Res<PlayerPosition>,
    q_zones: Query<&Zone>,
    mut q_cursor: Query<(&mut Position, &mut Visibility), With<ThrowCursor>>,
) {
    // Handle ESC key to cancel
    if is_key_pressed(KeyCode::Escape) {
        game_state.next = GameState::Inventory;
        return;
    }

    let Ok((mut cursor_pos, mut cursor_vis)) = q_cursor.single_mut() else {
        return;
    };

    let player_world = player_pos.world();
    let mouse_world = (
        mouse.world.0 as usize,
        mouse.world.1 as usize,
        player_world.2, // Same Z level as player
    );

    // Calculate distance from player to mouse position
    let dx = mouse_world.0 as i32 - player_world.0 as i32;
    let dy = mouse_world.1 as i32 - player_world.1 as i32;
    let distance = ((dx * dx + dy * dy) as f32).sqrt() as usize;

    // Check if mouse position is within throw range
    let in_range = distance <= context.throw_range;

    // Update cursor position and visibility
    cursor_pos.x = mouse_world.0 as f32;
    cursor_pos.y = mouse_world.1 as f32;
    cursor_pos.z = mouse_world.2 as f32;
    *cursor_vis = if in_range {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    // Handle mouse click to throw
    if is_mouse_button_pressed(MouseButton::Left) && in_range {
        // Check if target position is valid (same zone as player)
        let player_zone_idx = world_to_zone_idx(player_world.0, player_world.1, player_world.2);
        let target_zone_idx = world_to_zone_idx(mouse_world.0, mouse_world.1, mouse_world.2);

        if player_zone_idx == target_zone_idx {
            // Check if target zone is loaded
            let zone_loaded = q_zones.iter().any(|zone| zone.idx == target_zone_idx);

            if zone_loaded {
                // Execute throw action
                cmds.queue(ThrowItemAction {
                    thrower_entity: context.player_entity,
                    item_stable_id: context.item_id,
                    target_position: mouse_world,
                });

                game_state.next = GameState::Inventory;
            }
        }
    }
}

fn remove_throw_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<ThrowCallbacks>();
    cmds.remove_resource::<ThrowContext>();
}

pub struct ThrowStatePlugin;

impl Plugin for ThrowStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Throw)
            .on_enter(app, (setup_throw_callbacks, setup_throw_state).chain())
            .on_update(app, (game_loop, handle_throw_input).chain())
            .on_leave(
                app,
                (cleanup_system::<CleanupStateThrow>, remove_throw_callbacks).chain(),
            );
    }
}
