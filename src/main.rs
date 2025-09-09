use bevy_ecs::prelude::*;
use common::Palette;
use engine::{KeyInput, Time, render_fps, update_key_input, update_time};
use macroquad::{
    miniquad::conf::{Platform, WebGLVersion},
    prelude::*,
};
use rendering::{
    AnimatedGlyph, GameCamera, Layers, LightingData, Position, RenderTargets, ScreenSize, Text,
    render_all, render_glyphs, render_text, update_animated_glyphs, update_crt_uniforms,
    update_screen_size,
};
use ui::UiLayout;

use crate::{
    cfg::WINDOW_SIZE,
    common::Rand,
    domain::{
        ApplyVisibilityEffects, Bitmasker, Collider, Destructible, Energy, EquipmentSlots,
        Equippable, Equipped, GameSettings, Health, HideWhenNotVisible, InActiveZone, InInventory,
        Inventory, InventoryAccessible, IsExplored, IsVisible, Item, Label, LightSource,
        LoadGameResult, LoadZoneEvent, LootDrop, LootTableRegistry, MeleeWeapon, NeedsStableId,
        NewGameResult, Player, PlayerMovedEvent, Prefabs, RefreshBitmask, SaveFlag, SaveGameResult,
        SetZoneStatusEvent, StackCount, Stackable, StairDown, StairUp, TurnState, UnloadZoneEvent,
        UnopenedContainer, Vision, VisionBlocker, Zones, inventory::InventoryChangedEvent,
        on_bitmask_spawn, on_refresh_bitmask, systems::destruction_system::EntityDestroyedEvent,
    },
    engine::{
        App, Audio, Clock, ExitAppPlugin, FpsDisplay, Mouse, ScheduleType,
        SerializableComponentRegistry, StableId, StableIdRegistry, update_mouse,
        update_mouse_input,
    },
    rendering::{CrtShader, Glyph, RecordZonePosition, TilesetRegistry},
    states::{
        CleanupStateExplore, CleanupStatePlay, ContainerStatePlugin, CurrentAppState,
        CurrentGameState, DebugSpawnStatePlugin, ExploreStatePlugin, InventoryStatePlugin,
        LoadGameStatePlugin, MainMenuStatePlugin, NewGameStatePlugin, OverworldStatePlugin,
        PauseStatePlugin, PlayStatePlugin, SettingsStatePlugin, update_app_states,
        update_game_states,
    },
    ui::{
        DialogState, ListContext, UiFocus, clear_mouse_capture_when_not_hovering,
        hotkey_pressed_timer_system, list_cursor_visibility, list_mouse_wheel_scroll,
        selectable_list_interaction, setup_buttons, setup_lists, sync_focus_to_interaction,
        tab_navigation, ui_interaction_system, unified_click_system,
        unified_keyboard_activation_system, unified_style_system, update_focus_from_mouse,
        update_list_context,
    },
};

mod cfg;
mod common;
mod domain;
mod engine;
mod rendering;
mod states;
mod ui;

fn window_conf() -> Conf {
    Conf {
        window_title: "Quadboy".to_string(),
        window_width: WINDOW_SIZE.0 as i32,
        window_height: WINDOW_SIZE.1 as i32,
        fullscreen: false,
        window_resizable: true,
        platform: Platform {
            webgl_version: WebGLVersion::WebGL2,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    set_default_filter_mode(FilterMode::Nearest);

    let tileset_registry = TilesetRegistry::load().await;
    let audio_registry = Audio::load();

    let mut app = App::new();

    let mut reg = SerializableComponentRegistry::new();
    reg.register::<Position>();
    reg.register::<RecordZonePosition>();
    reg.register::<Glyph>();
    reg.register::<AnimatedGlyph>();
    reg.register::<SaveFlag>();
    reg.register::<CleanupStatePlay>();
    reg.register::<CleanupStateExplore>();
    reg.register::<Label>();
    reg.register::<Collider>();
    reg.register::<Energy>();
    reg.register::<StairDown>();
    reg.register::<StairUp>();
    reg.register::<InActiveZone>();
    reg.register::<Player>();
    reg.register::<Item>();
    reg.register::<Inventory>();
    reg.register::<InInventory>();
    reg.register::<StableId>();
    reg.register::<NeedsStableId>();
    reg.register::<Vision>();
    reg.register::<VisionBlocker>();
    reg.register::<IsVisible>();
    reg.register::<IsExplored>();
    reg.register::<ApplyVisibilityEffects>();
    reg.register::<HideWhenNotVisible>();
    reg.register::<InventoryAccessible>();
    reg.register::<EquipmentSlots>();
    reg.register::<Equippable>();
    reg.register::<Equipped>();
    reg.register::<Health>();
    reg.register::<Destructible>();
    reg.register::<MeleeWeapon>();
    reg.register::<UnopenedContainer>();
    reg.register::<LootDrop>();
    reg.register::<Stackable>();
    reg.register::<StackCount>();
    reg.register::<LightSource>();

    app.add_plugin(ExitAppPlugin)
        .add_plugin(MainMenuStatePlugin)
        .add_plugin(SettingsStatePlugin)
        .add_plugin(PlayStatePlugin)
        .add_plugin(NewGameStatePlugin)
        .add_plugin(LoadGameStatePlugin)
        .add_plugin(ExploreStatePlugin)
        .add_plugin(DebugSpawnStatePlugin)
        .add_plugin(InventoryStatePlugin)
        .add_plugin(ContainerStatePlugin::new())
        .add_plugin(OverworldStatePlugin)
        .add_plugin(PauseStatePlugin)
        .register_event::<LoadGameResult>()
        .register_event::<LoadZoneEvent>()
        .register_event::<NewGameResult>()
        .register_event::<UnloadZoneEvent>()
        .register_event::<SetZoneStatusEvent>()
        .register_event::<PlayerMovedEvent>()
        .register_event::<SaveGameResult>()
        .register_event::<RefreshBitmask>()
        .register_event::<EntityDestroyedEvent>()
        .register_event::<InventoryChangedEvent>()
        .insert_resource(tileset_registry)
        .insert_resource(audio_registry)
        .insert_resource(reg)
        .insert_resource(LootTableRegistry::new())
        .init_resource::<Mouse>()
        .init_resource::<ScreenSize>()
        .init_resource::<UiFocus>()
        .init_resource::<ListContext>()
        .init_resource::<Time>()
        .init_resource::<RenderTargets>()
        .init_resource::<Layers>()
        .init_resource::<KeyInput>()
        .init_resource::<CurrentAppState>()
        .init_resource::<CurrentGameState>()
        .init_resource::<GameCamera>()
        .init_resource::<UiLayout>()
        .init_resource::<DialogState>()
        .init_resource::<CrtShader>()
        .init_resource::<Zones>()
        .init_resource::<GameSettings>()
        .init_resource::<TurnState>()
        .init_resource::<Clock>()
        .init_resource::<Bitmasker>()
        .init_resource::<Prefabs>()
        .init_resource::<Rand>()
        .init_resource::<StableIdRegistry>()
        .init_resource::<LightingData>()
        .add_systems(
            ScheduleType::PreUpdate,
            (update_time, update_key_input, update_mouse_input),
        )
        .add_systems(
            ScheduleType::Update,
            (
                update_screen_size,
                update_mouse,
                ui_interaction_system,
                clear_mouse_capture_when_not_hovering,
                setup_buttons,
                list_mouse_wheel_scroll,
                setup_lists,
                tab_navigation,
                list_cursor_visibility,
                update_list_context,
                update_focus_from_mouse,
                sync_focus_to_interaction,
                selectable_list_interaction,
                unified_style_system,
                hotkey_pressed_timer_system,
                update_crt_uniforms,
                render_fps,
                (on_refresh_bitmask, on_bitmask_spawn).chain(),
            ),
        )
        .add_systems(
            ScheduleType::Update,
            (
                // New unified activation systems
                unified_keyboard_activation_system,
                unified_click_system,
            ),
        )
        .add_systems(
            ScheduleType::PostUpdate,
            (update_animated_glyphs, render_glyphs, render_text).chain(),
        )
        .add_systems(
            ScheduleType::FrameFinal,
            (
                render_all,
                // crate::engine::render_profiler,
            )
                .chain(),
        )
        .add_systems(
            ScheduleType::StateTransition,
            (update_app_states, update_game_states).chain(),
        );

    let world = app.get_world_mut();

    world.spawn((
        Text::new("123").bg(Palette::Black),
        Position::new_f32(0., 0., 0.),
        FpsDisplay,
    ));

    while app.run() {
        next_frame().await;
    }
}
