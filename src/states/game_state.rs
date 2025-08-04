use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

#[derive(Resource, Default)]
pub struct CurrentState {
    previous: Option<GameState>,
    current: GameState,
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
    move |res| res.current == state && res.next == state && res.previous == Some(state)
}

pub fn enter_state(state: GameState) -> impl Fn(Res<CurrentState>) -> bool {
    move |res| res.current == state && res.previous != Some(state)
}

pub fn leave_state(state: GameState) -> impl Fn(Res<CurrentState>) -> bool {
    move |res| res.current == state && res.next != state
}

pub fn update_states(mut state: ResMut<CurrentState>) {
    state.previous = Some(state.current);
    state.current = state.next;
}

pub fn cleanup_system<T: Component>(mut cmds: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        cmds.entity(e).despawn();
    }
}
