use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct CurrentState {
    pub previous: GameState,
    pub current: GameState,
    pub next: GameState,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

pub fn in_state(state: GameState) -> impl Fn(Res<CurrentState>) -> bool {
    move |res| res.current == state && res.next == state && res.previous == state
}

pub fn enter_state(state: GameState) -> impl Fn(Res<CurrentState>) -> bool {
    move |res| res.current == state && res.previous != state
}

pub fn leave_state(state: GameState) -> impl Fn(Res<CurrentState>) -> bool {
    move |res| res.current == state && res.next != state
}

pub fn update_states(mut state: ResMut<CurrentState>) {
    state.previous = state.current;
    state.current = state.next;
}
