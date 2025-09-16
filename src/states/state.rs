use bevy_ecs::{prelude::*, system::ScheduleSystem};
use macroquad::prelude::trace;
use std::fmt;

use crate::engine::{App, ScheduleType};

#[derive(Resource, Default)]
pub struct CurrentAppState {
    previous: Option<AppState>,
    current: AppState,
    pub next: AppState,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum AppState {
    #[default]
    MainMenu,
    Settings,
    Play,
}

pub fn in_app_state(state: AppState) -> impl Fn(Res<CurrentAppState>) -> bool {
    move |res| res.current == state && res.next == state && res.previous == Some(state)
}

pub fn enter_app_state(state: AppState) -> impl Fn(Res<CurrentAppState>) -> bool {
    move |res| res.current == state && res.previous != Some(state)
}

pub fn leave_app_state(state: AppState) -> impl Fn(Res<CurrentAppState>) -> bool {
    move |res| res.current == state && res.next != state
}

pub fn update_app_states(mut state: ResMut<CurrentAppState>) {
    state.previous = Some(state.current);
    state.current = state.next;
}

#[derive(Resource, Default)]
pub struct CurrentGameState {
    previous: Option<GameState>,
    current: GameState,
    pub next: GameState,
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum GameState {
    #[default]
    None,
    LoadGame,
    NewGame,
    Explore,
    Pause,
    Overworld,
    Inventory,
    Container,
    EquipSlotSelect,
    DebugSpawn,
    Attributes,
    GameOver,
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameState::None => write!(f, "None"),
            GameState::LoadGame => write!(f, "Load Game"),
            GameState::NewGame => write!(f, "New Game"),
            GameState::Explore => write!(f, "Explore"),
            GameState::Pause => write!(f, "Pause"),
            GameState::Overworld => write!(f, "Overworld"),
            GameState::Inventory => write!(f, "Inventory"),
            GameState::Container => write!(f, "Container"),
            GameState::EquipSlotSelect => write!(f, "Equip Slot Select"),
            GameState::DebugSpawn => write!(f, "Debug Spawn"),
            GameState::Attributes => write!(f, "Attributes"),
            GameState::GameOver => write!(f, "Game Over"),
        }
    }
}

pub fn in_game_state(state: GameState) -> impl Fn(Res<CurrentGameState>) -> bool {
    move |res| res.current == state && res.next == state && res.previous == Some(state)
}

pub fn enter_game_state(state: GameState) -> impl Fn(Res<CurrentGameState>) -> bool {
    move |res| res.current == state && res.previous != Some(state)
}

pub fn leave_game_state(state: GameState) -> impl Fn(Res<CurrentGameState>) -> bool {
    move |res| res.current == state && res.next != state
}

pub fn update_game_states(mut state: ResMut<CurrentGameState>) {
    state.previous = Some(state.current);
    state.current = state.next;
}

pub fn cleanup_system<T: Component>(mut cmds: Commands, q: Query<Entity, With<T>>) {
    trace!("running cleanup system!");

    for e in q.iter() {
        cmds.entity(e).despawn();
    }
}

pub struct GameStatePlugin {
    state: GameState,
}

impl GameStatePlugin {
    pub fn new(state: GameState) -> Self {
        Self { state }
    }

    pub fn on_enter<M>(
        &mut self,
        app: &mut App,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        app.add_systems(
            ScheduleType::PreUpdate,
            systems.run_if(enter_game_state(self.state)),
        );
        self
    }

    pub fn on_update<M>(
        &mut self,
        app: &mut App,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        app.add_systems(
            ScheduleType::Update,
            systems.run_if(in_game_state(self.state)),
        );
        self
    }

    pub fn on_leave<M>(
        &mut self,
        app: &mut App,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        app.add_systems(
            ScheduleType::FrameFinal,
            systems.run_if(leave_game_state(self.state)),
        );
        self
    }
}

pub struct AppStatePlugin {
    state: AppState,
}

impl AppStatePlugin {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn on_enter<M>(
        &mut self,
        app: &mut App,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        app.add_systems(
            ScheduleType::PreUpdate,
            systems.run_if(enter_app_state(self.state)),
        );
        self
    }

    pub fn on_update<M>(
        &mut self,
        app: &mut App,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        app.add_systems(
            ScheduleType::Update,
            systems.run_if(in_app_state(self.state)),
        );
        self
    }

    pub fn on_leave<M>(
        &mut self,
        app: &mut App,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        app.add_systems(
            ScheduleType::FrameFinal,
            systems.run_if(leave_app_state(self.state)),
        );
        self
    }
}
