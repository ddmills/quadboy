use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents different effects that can be applied when a hit lands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HitEffect {
    /// Knockback effect with a multiplier (typically Strength / 2)
    /// The multiplier is applied to the attacker's strength to determine knockback distance
    Knockback { strength: f32, chance: f32 },
    /// Poison effect with damage per tick and duration in ticks
    Poison {
        damage_per_tick: i32,
        duration_ticks: u32,
        chance: f32,
    },
    /// Bleeding effect with damage per tick and duration in ticks
    Bleeding {
        damage_per_tick: i32,
        duration_ticks: u32,
        chance: f32,
        can_stack: bool,
    },
    /// Burning effect with damage per tick and duration in ticks
    Burning {
        damage_per_tick: i32,
        duration_ticks: u32,
        chance: f32,
    },
}

/// Component for animating the visual knockback effect
/// The entity's Position is updated instantly, but this component
/// animates the Glyph's position_offset to smoothly transition from old to new position
#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct KnockbackAnimation {
    /// Initial offset (negative of knockback distance) - starts at old position relative to new position
    pub start_offset: (f32, f32),
    /// Time remaining for the animation
    pub duration_remaining: f32,
    /// Total duration of the animation
    pub total_duration: f32,
}

impl KnockbackAnimation {
    pub fn new(knockback_distance: (f32, f32), duration: f32) -> Self {
        Self {
            start_offset: (-knockback_distance.0, -knockback_distance.1),
            duration_remaining: duration,
            total_duration: duration,
        }
    }
}
