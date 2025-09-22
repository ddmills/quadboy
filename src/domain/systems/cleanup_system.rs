use super::destruction_system::EntityDestroyedEvent;
use crate::{
    common::Rand,
    domain::{Destructible, Player, Zone},
    engine::{Audio, Clock},
    states::{CurrentGameState, GameState},
};
use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;

#[profiled_system]
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
    for event in e_destroyed.read() {
        // Check if the destroyed entity is the player
        if q_player.contains(event.entity) {
            // Transition to game over state instead of despawning player
            game_state.next = GameState::GameOver;
            clock.force_update();
            continue;
        }

        for mut z in q_zones.iter_mut() {
            let _ = z.entities.remove(&event.entity);
            let _ = z.colliders.remove(&event.entity);
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
