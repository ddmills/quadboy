use super::destruction_system::EntityDestroyedEvent;
use bevy_ecs::prelude::*;

pub fn on_entity_destroyed_cleanup(
    mut e_destroyed: EventReader<EntityDestroyedEvent>,
    mut cmds: Commands,
) {
    for event in e_destroyed.read() {
        cmds.entity(event.entity).despawn();
    }
}
