use crate::{engine::SerializableComponent, rendering::world_to_zone_idx};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct PursuingPlayer {
    pub last_seen_at: (usize, usize, usize),
    pub target_zone: usize,
    pub pursuit_started: u32,
    pub waiting_to_teleport: bool,
    pub wait_started_tick: Option<u32>,
}

impl PursuingPlayer {
    pub fn new(player_position: (usize, usize, usize), current_tick: u32) -> Self {
        let target_zone = world_to_zone_idx(player_position.0, player_position.1, player_position.2);
        Self {
            last_seen_at: player_position,
            target_zone,
            pursuit_started: current_tick,
            waiting_to_teleport: false,
            wait_started_tick: None,
        }
    }

    pub fn update_last_seen(&mut self, player_position: (usize, usize, usize)) {
        self.last_seen_at = player_position;
        self.target_zone = world_to_zone_idx(player_position.0, player_position.1, player_position.2);
    }

    pub fn pursuit_duration(&self, current_tick: u32) -> u32 {
        current_tick.saturating_sub(self.pursuit_started)
    }

    pub const TELEPORT_WAIT_DURATION: u32 = 200;

    pub fn start_teleport_wait(&mut self, current_tick: u32) {
        self.waiting_to_teleport = true;
        self.wait_started_tick = Some(current_tick);
    }

    pub fn should_teleport(&self, current_tick: u32) -> bool {
        if let Some(wait_start) = self.wait_started_tick {
            current_tick.saturating_sub(wait_start) >= Self::TELEPORT_WAIT_DURATION
        } else {
            false
        }
    }

    pub fn reset_teleport_wait(&mut self) {
        self.waiting_to_teleport = false;
        self.wait_started_tick = None;
    }
}