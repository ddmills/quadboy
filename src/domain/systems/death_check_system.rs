use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;

use crate::{
    domain::{
        Health, Player,
        systems::destruction_system::{DestructionCause, EntityDestroyedEvent},
    },
    engine::StableIdRegistry,
    rendering::Position,
};

#[profiled_system]
pub fn death_check_system(
    q_entities: Query<(Entity, &Health, &Position, Option<&Player>)>,
    mut e_entity_destroyed: EventWriter<EntityDestroyedEvent>,
    stable_id_registry: Res<StableIdRegistry>,
) {

    for (entity, health, position, player_component) in q_entities.iter() {
        if health.is_dead() {
            let position_coords = position.world();

            let cause = if let Some(source_id) = health.last_damage_source {
                if let Some(source_entity) = stable_id_registry.get_entity(source_id.0) {
                    DestructionCause::Attack {
                        attacker: source_entity,
                    }
                } else {
                    DestructionCause::Environmental
                }
            } else {
                DestructionCause::Environmental
            };

            let event = EntityDestroyedEvent {
                entity,
                position: position_coords,
                cause,
                material_type: None, // Will be set by cleanup system if needed
            };

            e_entity_destroyed.write(event);
        }
    }
}
