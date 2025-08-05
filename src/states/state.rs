use bevy_ecs::prelude::*;

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
