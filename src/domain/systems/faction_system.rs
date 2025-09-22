use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;
use std::collections::HashMap;

use crate::{
    domain::{FactionId, FactionMember},
};

#[derive(Resource)]
pub struct FactionRelations {
    relations: HashMap<(FactionId, FactionId), i8>,
}

impl FactionRelations {
    pub fn new() -> Self {
        let mut relations = HashMap::new();

        // Initialize all factions as hostile to each other (-100)
        relations.insert((FactionId::Player, FactionId::Bandits), -100);
        relations.insert((FactionId::Bandits, FactionId::Player), -100);

        relations.insert((FactionId::Player, FactionId::Wildlife), -100);
        relations.insert((FactionId::Wildlife, FactionId::Player), -100);

        relations.insert((FactionId::Bandits, FactionId::Wildlife), -100);
        relations.insert((FactionId::Wildlife, FactionId::Bandits), -100);

        // Same faction relations (neutral)
        relations.insert((FactionId::Player, FactionId::Player), 0);
        relations.insert((FactionId::Bandits, FactionId::Bandits), 0);
        relations.insert((FactionId::Wildlife, FactionId::Wildlife), 0);

        Self { relations }
    }

    pub fn get_base_relationship(&self, faction_a: FactionId, faction_b: FactionId) -> i8 {
        self.relations
            .get(&(faction_a, faction_b))
            .copied()
            .unwrap_or(0)
    }

    pub fn set_relationship(&mut self, faction_a: FactionId, faction_b: FactionId, value: i8) {
        let clamped_value = value.clamp(-100, 100);
        self.relations.insert((faction_a, faction_b), clamped_value);
        self.relations.insert((faction_b, faction_a), clamped_value);
    }
}

pub fn get_effective_relationship(entity_a: Entity, entity_b: Entity, world: &World) -> i8 {
    let Some(faction_a) = world.get::<FactionMember>(entity_a) else {
        return 0;
    };

    let Some(faction_b) = world.get::<FactionMember>(entity_b) else {
        return 0;
    };

    let Some(faction_relations) = world.get_resource::<FactionRelations>() else {
        return 0;
    };

    let base_relationship =
        faction_relations.get_base_relationship(faction_a.faction_id, faction_b.faction_id);

    // Apply modifiers from entity_a's perspective
    let mut effective_relationship = base_relationship;

    // Check if entity_a is charmed (would make them friendly to enemies)
    if faction_a.has_modifier("charmed") && base_relationship < 0 {
        effective_relationship = 50; // Charmed makes hostile become friendly
    }

    // Check if entity_a is enraged (would make them hostile to everyone)
    if faction_a.has_modifier("enraged") && base_relationship >= 0 {
        effective_relationship = -75; // Enraged makes neutral/friendly become hostile
    }

    // Check if entity_a is feared (would make them avoid everyone)
    if faction_a.has_modifier("feared") {
        effective_relationship = -25; // Feared makes them want to avoid others
    }

    effective_relationship
}

pub fn are_hostile(entity_a: Entity, entity_b: Entity, world: &World) -> bool {
    get_effective_relationship(entity_a, entity_b, world) < 0
}

#[profiled_system]
pub fn tick_faction_modifiers(mut q_faction_members: Query<&mut FactionMember>) {
    for mut faction_member in q_faction_members.iter_mut() {
        faction_member.tick_modifiers();
    }
}

pub fn get_relationship_text(value: i8) -> &'static str {
    match value {
        -100..=-75 => "Hostile",
        -74..=-25 => "Unfriendly",
        -24..=24 => "Neutral",
        25..=74 => "Friendly",
        75..=100 => "Loyal",
        _ => "Unknown",
    }
}

pub fn get_relationship_color(value: i8) -> char {
    match value {
        -100..=-75 => 'R', // Red
        -74..=-25 => 'O',  // Orange
        -24..=24 => 'Y',   // Yellow
        25..=74 => 'G',    // Green
        75..=100 => 'B',   // Blue
        _ => 'w',          // White
    }
}

pub fn format_relationship_display(value: i8) -> String {
    let text = get_relationship_text(value);
    let color = get_relationship_color(value);
    format!("{{{color}|{text} ({value})}}")
}
