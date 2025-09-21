use bevy_ecs::prelude::*;
use common::Palette;
use engine::{KeyInput, Time, process_delayed_audio, render_fps, update_key_input, update_time};

#[derive(Resource, Default)]
pub struct DebugMode {
    pub ai_debug: bool,
}
use macroquad::{
    miniquad::conf::{Platform, WebGLVersion},
    prelude::*,
};
use rendering::{
    AmbientTransition, AnimatedGlyph, GameCamera, Layers, LightingData, ParticleGlyphPool,
    ParticleGrid, Position, RenderTargets, ScreenSize, Text, cleanup_particle_glyphs, render_all,
    render_glyphs, render_particle_fragments, render_text, update_animated_glyphs,
    update_crt_uniforms, update_particle_physics, update_particle_spawners, update_particle_trails,
    update_particles, update_screen_size,
};
use ui::UiLayout;

use crate::{
    cfg::WINDOW_SIZE,
    common::Rand,
    domain::{
        ActiveConditions, AiController, ApplyVisibilityEffects, AttributePoints, Attributes,
        Bitmasker, BumpAttack, Collider, Consumable, CreatureType, DefaultMeleeAttack, Description,
        Destructible, DynamicEntity, Energy, EquipmentSlots, Equippable, Equipped, ExplosionEvent,
        ExplosiveProperties, FactionMap, FactionMember, FactionRelations, Fuse, GameSettings,
        Health, HideWhenNotVisible, HitBlink, InActiveZone, InInventory, Inventory,
        InventoryAccessible, IsExplored, IsVisible, Item, ItemRarity, KnockbackAnimation, Label,
        Level, LightSource, LightStateChangedEvent, LoadGameResult, LoadZoneEvent, LootDrop,
        LootTableRegistry, MovementCapabilities, NeedsStableId, NewGameResult, Player,
        PlayerMovedEvent, Prefabs, PursuingTarget, RecalculateColliderFlagsEvent, RefreshBitmask,
        SaveFlag, SaveGameResult, SetZoneStatusEvent, SmoothMovement, StackCount, Stackable,
        StairDown, StairUp, StatModifiers, StaticEntity, StaticEntitySpawnedEvent, Stats,
        Throwable, TurnState, UnloadZoneEvent, UnopenedContainer, Vision, VisionBlocker, Weapon,
        Zones,
        inventory::InventoryChangedEvent,
        on_bitmask_spawn, on_refresh_bitmask,
        systems::bump_attack_system::bump_attack_system,
        systems::destruction_system::EntityDestroyedEvent,
        systems::dynamic_label_system::{
            ensure_labels_initialized, mark_dirty_on_equipment_change, mark_dirty_on_fuse_change,
            mark_dirty_on_light_change, mark_dirty_on_stack_change, update_labels,
        },
        systems::hit_blink_system::hit_blink_system,
        systems::knockback_animation_system::knockback_animation_system,
        systems::smooth_movement_system::smooth_movement_system,
        systems::xp_system::{
            LevelUpEvent, LevelUpParticleQueue, XPGainEvent, apply_xp_gain, award_xp_on_kill,
            handle_level_up, process_level_up_particles,
        },
    },
    engine::{
        App, Audio, Clock, ExitAppPlugin, FpsDisplay, Mouse, ScheduleType,
        SerializableComponentRegistry, StableId, StableIdRegistry, update_mouse,
        update_mouse_input,
    },
    rendering::{CrtShader, Glyph, TilesetRegistry},
    states::{
        AttributesStatePlugin, CleanupStateExplore, CleanupStatePlay, ContainerStatePlugin,
        CurrentAppState, CurrentGameState, DebugSpawnStatePlugin, ExploreStatePlugin,
        GameOverStatePlugin, InventoryStatePlugin, LoadGameStatePlugin, MainMenuStatePlugin,
        NewGameStatePlugin, OverworldStatePlugin, PauseStatePlugin, PlayStatePlugin,
        SettingsStatePlugin, ThrowStatePlugin, update_app_states, update_game_states,
    },
    ui::{
        DialogState, ListContext, UiFocus, clear_mouse_capture_when_not_hovering,
        hotkey_pressed_timer_system, list_cursor_visibility, list_mouse_wheel_scroll,
        render_dialog_content, selectable_list_interaction, setup_buttons, setup_dialogs,
        setup_lists, sync_focus_to_interaction, tab_navigation, ui_interaction_system,
        unified_click_system, unified_keyboard_activation_system, unified_style_system,
        update_focus_from_mouse, update_list_context,
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
    #[cfg(feature = "tracy")]
    tracy_client::Client::start();

    set_default_filter_mode(FilterMode::Nearest);

    let tileset_registry = TilesetRegistry::load().await;
    let audio_registry = Audio::load();

    let mut app = App::new();

    let mut reg = SerializableComponentRegistry::new();
    reg.register::<Position>();
    reg.register::<StaticEntity>();
    reg.register::<DynamicEntity>();
    reg.register::<Glyph>();
    reg.register::<AnimatedGlyph>();
    reg.register::<SaveFlag>();
    reg.register::<CleanupStatePlay>();
    reg.register::<CleanupStateExplore>();
    reg.register::<Label>();
    reg.register::<Description>();
    reg.register::<Collider>();
    reg.register::<MovementCapabilities>();
    reg.register::<Consumable>();
    reg.register::<Energy>();
    reg.register::<ActiveConditions>();
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
    reg.register::<HitBlink>();
    reg.register::<KnockbackAnimation>();
    reg.register::<BumpAttack>();
    reg.register::<SmoothMovement>();
    reg.register::<Destructible>();
    reg.register::<Weapon>();
    reg.register::<ItemRarity>();
    reg.register::<DefaultMeleeAttack>();
    reg.register::<CreatureType>();
    reg.register::<AiController>();
    reg.register::<Level>();
    reg.register::<Attributes>();
    reg.register::<AttributePoints>();
    reg.register::<Stats>();
    reg.register::<StatModifiers>();
    reg.register::<UnopenedContainer>();
    reg.register::<LootDrop>();
    reg.register::<Stackable>();
    reg.register::<StackCount>();
    reg.register::<Throwable>();
    reg.register::<ExplosiveProperties>();
    reg.register::<Fuse>();
    reg.register::<LightSource>();
    reg.register::<FactionMember>();
    reg.register::<PursuingTarget>();

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
        .add_plugin(ThrowStatePlugin)
        .add_plugin(AttributesStatePlugin)
        .add_plugin(OverworldStatePlugin)
        .add_plugin(PauseStatePlugin)
        .add_plugin(GameOverStatePlugin)
        .register_event::<LoadGameResult>()
        .register_event::<LoadZoneEvent>()
        .register_event::<NewGameResult>()
        .register_event::<UnloadZoneEvent>()
        .register_event::<SetZoneStatusEvent>()
        .register_event::<PlayerMovedEvent>()
        .register_event::<SaveGameResult>()
        .register_event::<RefreshBitmask>()
        .register_event::<RecalculateColliderFlagsEvent>()
        .register_event::<StaticEntitySpawnedEvent>()
        .register_event::<EntityDestroyedEvent>()
        .register_event::<XPGainEvent>()
        .register_event::<LevelUpEvent>()
        .register_event::<InventoryChangedEvent>()
        .register_event::<LightStateChangedEvent>()
        .register_event::<ExplosionEvent>()
        .insert_resource(tileset_registry)
        .insert_resource(audio_registry)
        .insert_resource(reg)
        .insert_resource(LootTableRegistry::new())
        .init_resource::<LevelUpParticleQueue>()
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
        .insert_resource(FactionMap::new())
        .init_resource::<LightingData>()
        .init_resource::<AmbientTransition>()
        .init_resource::<ParticleGrid>()
        .init_resource::<ParticleGlyphPool>()
        .insert_resource(FactionRelations::new())
        .init_resource::<DebugMode>()
        .add_systems(
            ScheduleType::PreUpdate,
            (
                update_time,
                update_key_input,
                update_mouse_input,
                process_delayed_audio,
            ),
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
                setup_dialogs,
                render_dialog_content,
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
                // Label systems
                ensure_labels_initialized,
                (
                    mark_dirty_on_equipment_change,
                    mark_dirty_on_fuse_change,
                    mark_dirty_on_light_change,
                    mark_dirty_on_stack_change,
                )
                    .after(ensure_labels_initialized),
                update_labels.after(ensure_labels_initialized),
            ),
        )
        .add_systems(
            ScheduleType::PostUpdate,
            (
                bump_attack_system,
                smooth_movement_system,
                hit_blink_system,
                knockback_animation_system,
                (award_xp_on_kill, apply_xp_gain, handle_level_up).chain(),
                process_level_up_particles,
                update_animated_glyphs,
                update_particle_physics,
                update_particle_spawners,
                update_particle_trails,
                update_particles,
                render_particle_fragments,
                render_glyphs,
                render_text,
                cleanup_particle_glyphs,
            )
                .chain(),
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
        tracy_frame_mark!();
        next_frame().await;
    }
}
