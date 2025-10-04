use bevy_ecs::prelude::*;

#[allow(dead_code)]
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
}
