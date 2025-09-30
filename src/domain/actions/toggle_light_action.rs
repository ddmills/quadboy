use bevy_ecs::prelude::*;

use crate::{
    common::Palette,
    domain::{
        Energy, EnergyActionType, ExplosiveProperties, Fuse, HitBlink, LightSource, Lightable,
        PlayerPosition, actions::GameAction, get_base_energy_cost, split_item_from_stack,
    },
    engine::{Audio, Clock, StableId, StableIdRegistry},
    rendering::Position,
};

#[derive(Event)]
pub struct LightStateChangedEvent {
    pub item_id: u64,
}

impl LightStateChangedEvent {
    pub fn new(item_id: u64) -> Self {
        Self { item_id }
    }
}

pub struct ToggleLightAction {
    pub item_id: u64,  // Stable ID of the item to toggle
    pub actor: Entity, // Entity that is doing the action
}

impl ToggleLightAction {
    pub fn new(item_id: u64, actor: Entity) -> Self {
        Self { item_id, actor }
    }
}

impl GameAction for ToggleLightAction {
    fn try_apply(self, world: &mut World) -> bool {
        // Get the item entity from stable ID
        let item_entity = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return false;
            };
            let Some(entity) = registry.get_entity(StableId(self.item_id)) else {
                return false;
            };
            entity
        };

        // Track which item actually gets lit (might be different due to stack splitting)
        let mut final_lit_item_id = self.item_id;

        // Check if this is an explosive item (has ExplosiveProperties)
        let is_explosive = world.get::<ExplosiveProperties>(item_entity).is_some();

        if is_explosive {
            // Handle explosive lighting/extinguishing
            let has_fuse = world.get::<Fuse>(item_entity).is_some();

            if has_fuse {
                // Extinguish the fuse
                world.entity_mut(item_entity).remove::<Fuse>();

                // Remove blinking effect
                world.entity_mut(item_entity).remove::<HitBlink>();

                // Update label and play extinguish audio if available
                if let Some(mut lightable) = world.get_mut::<Lightable>(item_entity) {
                    lightable.update_label(false);

                    // Play extinguish audio if available
                    if let Some(extinguish_audio) = lightable.extinguish_audio
                        && let Some(audio) = world.get_resource::<Audio>()
                        && let Some(player_pos) = world.get_resource::<PlayerPosition>()
                        && let Some(item_pos) = world.get::<Position>(item_entity)
                    {
                        audio.play_at_position(extinguish_audio, 0.6, item_pos.world(), player_pos);
                    }
                }
            } else {
                // Light the fuse - but first split from stack if needed
                if let Some(explosive_props) = world.get::<ExplosiveProperties>(item_entity) {
                    // Extract values before splitting
                    let fuse_duration = explosive_props.fuse_duration;
                    let radius = explosive_props.radius;
                    let base_damage = explosive_props.base_damage;

                    // Split item from stack if it's part of a stack
                    let actual_item_entity = split_item_from_stack(world, item_entity, self.actor)
                        .unwrap_or(item_entity);

                    let current_tick = world
                        .get_resource::<Clock>()
                        .map(|c| c.get_tick())
                        .unwrap_or(0);

                    let fuse = Fuse::new(fuse_duration, radius, base_damage, current_tick);

                    world.entity_mut(actual_item_entity).insert(fuse);

                    // Add blinking effect for lit dynamite
                    let hit_blink = HitBlink::blinking(Palette::White.into(), 2.0);
                    world.entity_mut(actual_item_entity).insert(hit_blink);

                    // Update label and play light audio if available
                    if let Some(mut lightable) = world.get_mut::<Lightable>(actual_item_entity) {
                        lightable.update_label(true);

                        // Play light audio if available
                        if let Some(light_audio) = lightable.light_audio
                            && let Some(audio) = world.get_resource::<Audio>()
                            && let Some(player_pos) = world.get_resource::<PlayerPosition>()
                            && let Some(item_pos) = world.get::<Position>(item_entity)
                        {
                            audio.play_at_position(light_audio, 0.6, item_pos.world(), player_pos);
                        }
                    }

                    // Update the item_id to point to the lit item for the event
                    if let Some(stable_id) =
                        world.get::<crate::engine::StableId>(actual_item_entity)
                    {
                        final_lit_item_id = stable_id.0;
                    }
                }
            }
        } else {
            // Handle normal light source items
            // Check if the item has both Lightable and LightSource components
            let has_components = world.get::<Lightable>(item_entity).is_some()
                && world.get::<LightSource>(item_entity).is_some();

            if !has_components {
                return false;
            }

            // Toggle the light source
            let is_enabled =
                if let Some(mut light_source) = world.get_mut::<LightSource>(item_entity) {
                    light_source.is_enabled = !light_source.is_enabled;
                    light_source.is_enabled
                } else {
                    return false;
                };

            // Get position and player position first to avoid borrow checker issues
            let item_pos = world.get::<Position>(item_entity).map(|p| p.world());
            let player_pos = world.get_resource::<PlayerPosition>().cloned();

            // Update the lightable component's action label and play audio
            if let Some(mut lightable) = world.get_mut::<Lightable>(item_entity) {
                let light_audio = lightable.light_audio;
                let extinguish_audio = lightable.extinguish_audio;
                lightable.update_label(is_enabled);

                // Play appropriate audio if available
                if let Some(player_pos) = player_pos
                    && let Some(item_pos) = item_pos
                    && let Some(audio) = world.get_resource::<Audio>()
                {
                    if is_enabled {
                        if let Some(light_audio) = light_audio {
                            audio.play_at_position(light_audio, 0.6, item_pos, &player_pos);
                        }
                    } else if let Some(extinguish_audio) = extinguish_audio {
                        audio.play_at_position(extinguish_audio, 0.6, item_pos, &player_pos);
                    }
                }
            }
        }

        if let Some(mut energy) = world.get_mut::<Energy>(self.actor) {
            let cost = get_base_energy_cost(EnergyActionType::ToggleLight);
            energy.consume_energy(cost);
        }

        // Send event to notify that light state has changed
        world.send_event(LightStateChangedEvent::new(final_lit_item_id));

        true
    }
}

impl Command for ToggleLightAction {
    fn apply(self, world: &mut World) {
        self.try_apply(world);
    }
}
