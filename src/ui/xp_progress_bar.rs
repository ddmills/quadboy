use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{Level, Player},
    engine::SerializableComponent,
    rendering::Text,
    ui::Bar,
};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct XPProgressBar {
    pub width: u32,
}

impl XPProgressBar {
    pub fn new(width: u32) -> Self {
        Self { width }
    }
}

pub fn update_xp_progress_bars(
    mut q_bars: Query<(&XPProgressBar, &mut Text, &mut Bar)>,
    q_player_level: Query<&Level, With<Player>>,
) {
    let Ok(player_level) = q_player_level.single() else {
        return;
    };

    for (xp_bar, mut text, mut bar) in q_bars.iter_mut() {
        bar.update_values(player_level.current_xp as usize, player_level.xp_to_next_level as usize);
        let new_text = format!("Level {}", player_level.current_level);
        if text.value != new_text {
            text.value = new_text;
        }
    }
}

fn generate_xp_display(bar: &XPProgressBar, level: &Level) -> String {
    let progress = level.xp_progress_percentage();
    let filled_chars = (bar.width as f32 * progress).round() as u32;
    let empty_chars = bar.width - filled_chars;

    format!(
        "Level {} [{{G|{}}}{{x|{}}}] XP: {}/{}",
        level.current_level,
        "½".repeat(filled_chars as usize),
        " ".repeat(empty_chars as usize),
        level.current_xp,
        level.xp_to_next_level
    )
}
