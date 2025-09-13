use crate::domain::{
    GameFormulas, Level,
    systems::destruction_system::{DestructionCause, EntityDestroyedEvent},
};
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct XPGainEvent {
    pub recipient_entity: Entity,
    pub xp_amount: u32,
    pub source_entity: Entity,
    pub source_description: String,
}

/// Award XP to any entity with a Level component when they kill another leveled entity
pub fn award_xp_on_kill(
    mut e_entity_destroyed: EventReader<EntityDestroyedEvent>,
    mut e_xp_gain: EventWriter<XPGainEvent>,
    q_levels: Query<&Level>,
) {
    for destroyed_event in e_entity_destroyed.read() {
        // Only process attack-based deaths
        if let DestructionCause::Attack { attacker } = destroyed_event.cause {
            // Check if attacker has Level component (can gain XP)
            if let Ok(attacker_level) = q_levels.get(attacker) {
                // Check if destroyed entity had Level component (gives XP)
                if let Ok(victim_level) = q_levels.get(destroyed_event.entity) {
                    let xp_gained = GameFormulas::calculate_xp_gain(
                        attacker_level.current_level,
                        victim_level.current_level,
                    );

                    e_xp_gain.write(XPGainEvent {
                        recipient_entity: attacker,
                        xp_amount: xp_gained,
                        source_entity: destroyed_event.entity,
                        source_description: format!(
                            "Defeated Level {} Enemy",
                            victim_level.current_level
                        ),
                    });
                }
            }
        }
    }
}

/// Apply XP gains to entities with Level components
pub fn apply_xp_gain(mut e_xp_gain: EventReader<XPGainEvent>, mut q_levels: Query<&mut Level>) {
    for xp_event in e_xp_gain.read() {
        if let Ok(mut level) = q_levels.get_mut(xp_event.recipient_entity) {
            let old_level = level.current_level;
            let leveled_up = level.add_xp(xp_event.xp_amount);

            if leveled_up {
                println!(
                    "Entity leveled up from {} to {}! (+{} XP)",
                    old_level, level.current_level, xp_event.xp_amount
                );
            } else {
                println!("Entity gained {} XP", xp_event.xp_amount);
            }
        }
    }
}
