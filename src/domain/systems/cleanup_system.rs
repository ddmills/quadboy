use super::destruction_system::EntityDestroyedEvent;
use crate::{common::Rand, domain::Destructible, engine::AudioRegistry};
use bevy_ecs::prelude::*;

pub fn on_entity_destroyed_cleanup(
    mut e_destroyed: EventReader<EntityDestroyedEvent>,
    q_destructible: Query<&Destructible>,
    audio_registry: Option<Res<AudioRegistry>>,
    mut rand: Option<ResMut<Rand>>,
    mut cmds: Commands,
) {
    for event in e_destroyed.read() {
        // Play destruction audio if the entity has a destructible component
        if let Ok(destructible) = q_destructible.get(event.entity)
            && let Some(audio_collection) = destructible.material_type.destroy_audio_collection()
            && let (Some(audio_registry), Some(rand)) = (&audio_registry, &mut rand)
        {
            audio_registry.play_random_from_collection(audio_collection, rand, 0.7);
        }

        cmds.entity(event.entity).despawn();
    }
}
