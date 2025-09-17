use super::destruction_system::EntityDestroyedEvent;
use crate::{
    common::Rand,
    domain::{Destructible, Player, Zone},
    engine::{Audio, Clock},
    states::{CurrentGameState, GameState},
    tracy_span,
};
use bevy_ecs::prelude::*;

pub fn on_entity_destroyed_cleanup(
    mut e_destroyed: EventReader<EntityDestroyedEvent>,
    q_destructible: Query<&Destructible>,
    q_player: Query<&Player>,
    mut q_zones: Query<&mut Zone>,
    audio_registry: Option<Res<Audio>>,
    mut rand: Option<ResMut<Rand>>,
    mut cmds: Commands,
    mut clock: ResMut<Clock>,
    mut game_state: ResMut<CurrentGameState>,
) {
    tracy_span!("on_entity_destroyed_cleanup");
    for event in e_destroyed.read() {
        // Check if the destroyed entity is the player
        if q_player.contains(event.entity) {
            // Transition to game over state instead of despawning player
            game_state.next = GameState::GameOver;
            clock.force_update();
            continue;
        }

        for mut z in q_zones.iter_mut() {
            z.entities.remove(&event.entity);
            z.colliders.remove(&event.entity);
        }

        // Play destruction audio if the entity has a destructible component
        if let Ok(destructible) = q_destructible.get(event.entity)
            && let Some(audio_collection) = destructible.material_type.destroy_audio_collection()
            && let (Some(audio_registry), Some(rand)) = (&audio_registry, &mut rand)
        {
            audio_registry.play_random_from_collection(audio_collection, rand, 0.7);
        }

        cmds.entity(event.entity).despawn();
        clock.force_update();
    }
}
