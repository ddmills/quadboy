use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;
use std::collections::VecDeque;

use crate::{
    domain::{Label, Player, Zone, Zones},
    engine::Clock,
    rendering::{Position, Visibility, world_to_zone_idx, world_to_zone_local},
};

#[derive(Event)]
pub struct GameLogEvent {
    pub message: LogMessage,
    pub tick: u32,
    pub knowledge: KnowledgeLevel,
}

#[derive(Clone)]
pub enum LogMessage {
    // Combat
    Attack {
        attacker: Entity,
        target: Entity,
        damage: i32,
    },
    Death {
        entity: Entity,
        killer: Option<Entity>,
    },

    // Status Effects
    PoisonApplied {
        source: Entity,
        target: Entity,
    },
    BleedingApplied {
        source: Entity,
        target: Entity,
    },
    BurningApplied {
        target: Entity,
    },

    // Items
    ItemPickup {
        picker: Entity,
        item: Entity,
        quantity: Option<u32>,
    },
    ItemDrop {
        dropper: Entity,
        item: Entity,
        quantity: Option<u32>,
    },

    // Progression
    XpGain {
        entity: Entity,
        amount: u32,
        source: Entity,
    },
    LevelUp {
        entity: Entity,
        new_level: u32,
    },

    // Environmental/System
    Discovery {
        text: String,
    },
    GameSaved,
    GameLoaded,
    Custom(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum KnowledgeLevel {
    /// Always visible - player's own actions
    Player,

    /// Visible only if actor/target is in sight
    Action {
        actor: Entity,
        location: (usize, usize, usize),
    },

    /// Always visible - world events, system messages
    Global,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LogCategory {
    Combat,
    Status,
    Item,
    Progression,
    Discovery,
    System,
}

impl LogMessage {
    pub fn category(&self) -> LogCategory {
        match self {
            LogMessage::Attack { .. } | LogMessage::Death { .. } => LogCategory::Combat,
            LogMessage::PoisonApplied { .. }
            | LogMessage::BleedingApplied { .. }
            | LogMessage::BurningApplied { .. } => LogCategory::Status,
            LogMessage::ItemPickup { .. } | LogMessage::ItemDrop { .. } => LogCategory::Item,
            LogMessage::XpGain { .. } | LogMessage::LevelUp { .. } => LogCategory::Progression,
            LogMessage::Discovery { .. } => LogCategory::Discovery,
            LogMessage::GameSaved | LogMessage::GameLoaded | LogMessage::Custom(_) => {
                LogCategory::System
            }
        }
    }
}

#[derive(Resource)]
pub struct GameLog {
    messages: VecDeque<LogEntry>,
    max_messages: usize,
    new_message_count: usize,
}

#[derive(Clone)]
pub struct LogEntry {
    pub text: String,
    pub tick: u32,
    pub category: LogCategory,
}

impl Default for GameLog {
    fn default() -> Self {
        Self::new()
    }
}

impl GameLog {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 100,
            new_message_count: 0,
        }
    }

    pub fn add_message(&mut self, entry: LogEntry) {
        if self.messages.len() >= self.max_messages {
            self.messages.pop_front();
        }
        self.messages.push_back(entry);
        self.new_message_count += 1;
    }

    pub fn get_messages(&self) -> &VecDeque<LogEntry> {
        &self.messages
    }

    pub fn get_recent_messages(&self, count: usize) -> Vec<&LogEntry> {
        self.messages.iter().rev().take(count).rev().collect()
    }

    pub fn mark_messages_read(&mut self) {
        self.new_message_count = 0;
    }

    pub fn has_new_messages(&self) -> bool {
        self.new_message_count > 0
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.new_message_count = 0;
    }
}

#[profiled_system]
pub fn process_game_log_events(
    mut e_log: EventReader<GameLogEvent>,
    mut game_log: ResMut<GameLog>,
    q_labels: Query<&Label>,
    q_player: Query<(&Player, &Position)>,
    q_zones: Query<&Zone>,
    zones: Res<Zones>,
) {
    // Get player position for visibility checks
    let player_pos = q_player
        .iter()
        .next()
        .map(|(_, pos)| (pos.x as usize, pos.y as usize, pos.z as usize));

    for event in e_log.read() {
        // Check if we should show this message based on knowledge level
        let should_show = match event.knowledge {
            KnowledgeLevel::Player => true,
            KnowledgeLevel::Global => true,
            KnowledgeLevel::Action { actor: _, location } => {
                is_action_visible(location, player_pos, &q_zones, &zones)
            }
        };

        if !should_show {
            continue; // Skip messages player shouldn't see
        }

        // Format and store the message
        let message_text = format_log_message(&event.message, &q_labels, &q_player);

        game_log.add_message(LogEntry {
            text: message_text,
            tick: event.tick,
            category: event.message.category(),
        });
    }
}

fn is_action_visible(
    location: (usize, usize, usize),
    player_pos: Option<(usize, usize, usize)>,
    q_zones: &Query<&Zone>,
    zones: &Zones,
) -> bool {
    let Some(player_pos) = player_pos else {
        return false;
    };

    // Same zone check
    if location.2 != player_pos.2 {
        return false;
    }

    // Get the zone index for the action location
    let zone_idx = world_to_zone_idx(location.0, location.1, location.2);

    // Get the zone entity from the cache
    let Some(zone_entity) = zones.cache.get(&zone_idx) else {
        return false;
    };

    // Get the zone data
    let Ok(zone) = q_zones.get(*zone_entity) else {
        return false;
    };

    // Convert world coordinates to local zone coordinates
    let (local_x, local_y) = world_to_zone_local(location.0, location.1);

    // Check if this location is currently visible in the zone's visibility grid
    zone.visible.get(local_x, local_y).copied().unwrap_or(false)
}

fn format_log_message(
    message: &LogMessage,
    q_labels: &Query<&Label>,
    q_player: &Query<(&Player, &Position)>,
) -> String {
    match message {
        LogMessage::Attack {
            attacker,
            target,
            damage,
        } => {
            let attacker_label = get_entity_label(*attacker, q_labels, q_player);
            let target_label = get_entity_label(*target, q_labels, q_player);
            format!(
                "{} hits {} for {{r|{} damage}}!",
                attacker_label, target_label, damage
            )
        }

        LogMessage::Death { entity, killer } => {
            let entity_label = get_entity_label(*entity, q_labels, q_player);
            match killer {
                Some(k) => {
                    let killer_label = get_entity_label(*k, q_labels, q_player);
                    format!("{} was slain by {}!", entity_label, killer_label)
                }
                None => format!("{} died.", entity_label),
            }
        }

        LogMessage::PoisonApplied { source, target } => {
            let source_label = get_entity_label(*source, q_labels, q_player);
            let is_player_target = q_player.get(*target).is_ok();

            if is_player_target {
                format!("{} poisons {{C|you}}! {{g|You feel sick.}}", source_label)
            } else {
                let target_label = get_entity_label(*target, q_labels, q_player);
                format!("{} poisons {}!", source_label, target_label)
            }
        }

        LogMessage::BleedingApplied { source, target } => {
            let source_label = get_entity_label(*source, q_labels, q_player);
            let is_player_target = q_player.get(*target).is_ok();

            if is_player_target {
                format!("{} wounds {{C|you}}! {{r|You are bleeding!}}", source_label)
            } else {
                let target_label = get_entity_label(*target, q_labels, q_player);
                format!("{} wounds {}!", source_label, target_label)
            }
        }

        LogMessage::BurningApplied { target } => {
            let is_player_target = q_player.get(*target).is_ok();

            if is_player_target {
                "{O|Fire} spreads to {{C|you}}! {{o|You are burning!}}".to_string()
            } else {
                let target_label = get_entity_label(*target, q_labels, q_player);
                format!("{{O|Fire}} spreads to {}!", target_label)
            }
        }

        LogMessage::ItemPickup {
            picker,
            item,
            quantity,
        } => {
            let picker_label = get_entity_label(*picker, q_labels, q_player);
            let item_label = get_entity_label(*item, q_labels, q_player);

            match quantity {
                Some(n) if *n > 1 => format!("{} pick up {} {}.", picker_label, n, item_label),
                _ => format!("{} pick up {}.", picker_label, item_label),
            }
        }

        LogMessage::ItemDrop {
            dropper,
            item,
            quantity,
        } => {
            let dropper_label = get_entity_label(*dropper, q_labels, q_player);
            let item_label = get_entity_label(*item, q_labels, q_player);

            match quantity {
                Some(n) if *n > 1 => format!("{} drop {} {}.", dropper_label, n, item_label),
                _ => format!("{} drop {}.", dropper_label, item_label),
            }
        }

        LogMessage::XpGain {
            entity,
            amount,
            source,
        } => {
            let entity_label = get_entity_label(*entity, q_labels, q_player);
            let source_label = get_entity_label(*source, q_labels, q_player);
            format!(
                "{} defeated {}! {{g|(+{} XP)}}",
                entity_label, source_label, amount
            )
        }

        LogMessage::LevelUp { entity, new_level } => {
            let entity_label = get_entity_label(*entity, q_labels, q_player);
            format!(
                "{{G|Level up!}} {} are now level {{C|{}}}!",
                entity_label, new_level
            )
        }

        // Environmental and text-based messages
        LogMessage::Discovery { text } => text.clone(),
        LogMessage::GameSaved => "{B|Game saved.}".to_string(),
        LogMessage::GameLoaded => "{B|Game loaded.}".to_string(),
        LogMessage::Custom(text) => text.clone(),
    }
}

fn get_entity_label(
    entity: Entity,
    q_labels: &Query<&Label>,
    q_player: &Query<(&Player, &Position)>,
) -> String {
    // Check if it's the player
    // if q_player.get(entity).is_ok() {
    //     return "{C|You}".to_string();
    // }

    // Get the label, with fallback
    q_labels
        .get(entity)
        .map(|label| label.get().to_string())
        .unwrap_or_else(|_| "{x|something}".to_string())
}
