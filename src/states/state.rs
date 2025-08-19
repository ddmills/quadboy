use bevy_ecs::{prelude::*, system::ScheduleSystem};

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

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    #[default]
    None,
    LoadGame,
    NewGame,
    Explore,
    Pause,
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
            ScheduleType::PostUpdate,
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
            ScheduleType::PostUpdate,
            systems.run_if(leave_app_state(self.state)),
        );
        self
    }
}
