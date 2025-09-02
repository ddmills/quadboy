use bevy_ecs::prelude::*;

use crate::{
    common::Rand,
    domain::{
        Destructible, Energy, EnergyActionType, EquipmentSlots, Health, LootDrop,
        LootTableRegistry, MeleeWeapon, Prefab, Prefabs, Zone, get_energy_cost,
    },
    engine::StableIdRegistry,
    rendering::Position,
};

pub struct AttackAction {
    pub attacker_entity: Entity,
    pub target_pos: (usize, usize, usize),
}

fn spawn_loot_drops(world: &mut World, entity: Entity) {
    // Clone the values we need to avoid borrow conflicts
    let loot_drop = if let Some(loot_drop) = world.get::<LootDrop>(entity) {
        loot_drop.clone()
    } else {
        return;
    };

    let position_coords = if let Some(position) = world.get::<Position>(entity) {
        position.world()
    } else {
        return;
    };

    // Use resource_scope to safely access both resources together
    let items_to_spawn = world.resource_scope(|world, loot_registry: Mut<LootTableRegistry>| {
        if let Some(mut rand) = world.get_resource_mut::<Rand>() {
            // Roll for drop chance
            if rand.bool(loot_drop.drop_chance) {
                // Success! Roll for loot items
                loot_registry.roll_multiple(loot_drop.loot_table, loot_drop.drop_count, &mut rand)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    });

    // Spawn each item at the position
    for item_id in items_to_spawn {
        let config = Prefab::new(item_id, position_coords);
        Prefabs::spawn_world(world, config);
    }
}

impl Command for AttackAction {
    fn apply(self, world: &mut World) {
        // Get the equipped weapon (if any)
        let weapon_damage = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return;
            };

            let Some(attacker_id) = registry.get_id(self.attacker_entity) else {
                return;
            };

            let Some(equipment) = world.get::<EquipmentSlots>(self.attacker_entity) else {
                return; // No equipment slots, can't have weapon
            };

            // Check for equipped weapon in MainHand
            let weapon_id = equipment.get_equipped_item(crate::domain::EquipmentSlot::MainHand);

            if let Some(weapon_id) = weapon_id {
                if let Some(weapon_entity) = registry.get_entity(weapon_id) {
                    if let Some(weapon) = world.get::<MeleeWeapon>(weapon_entity) {
                        Some((weapon.damage, weapon.can_damage.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Find target at position
        let targets = {
            let zone_idx = crate::rendering::world_to_zone_idx(
                self.target_pos.0,
                self.target_pos.1,
                self.target_pos.2,
            );
            let local = crate::rendering::world_to_zone_local(self.target_pos.0, self.target_pos.1);

            let mut found_targets = Vec::new();
            let mut zones = world.query::<&Zone>();
            for zone in zones.iter(world) {
                if zone.idx == zone_idx {
                    if let Some(entities) = zone.entities.get(local.0, local.1) {
                        found_targets.extend(entities.iter().copied());
                    }
                    break;
                }
            }
            found_targets
        };

        if targets.is_empty() {
            return;
        }

        // Process attack on each target at position
        for &target_entity in targets.iter() {
            // Check if target has Health (living creature)
            if let Some(mut health) = world.get_mut::<Health>(target_entity) {
                if let Some((damage, can_damage)) = &weapon_damage {
                    // Check if weapon can damage flesh
                    if can_damage.contains(&crate::domain::MaterialType::Flesh) {
                        health.take_damage(*damage);

                        // Check if target died
                        if health.is_dead() {
                            spawn_loot_drops(world, target_entity);
                            world.entity_mut(target_entity).despawn();
                        }
                    }
                }
            }
            // Check if target has Destructible (object)
            else if let Some(mut destructible) = world.get_mut::<Destructible>(target_entity) {
                if let Some((damage, can_damage)) = &weapon_damage {
                    // Check if weapon can damage this material type
                    if can_damage.contains(&destructible.material_type) {
                        destructible.take_damage(*damage);

                        // Check if target was destroyed
                        if destructible.is_destroyed() {
                            spawn_loot_drops(world, target_entity);
                            world.entity_mut(target_entity).despawn();
                        }
                    }
                }
            }
        }

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(self.attacker_entity) {
            let cost = get_energy_cost(EnergyActionType::Move); // Same cost as movement for now
            energy.consume_energy(cost);
        }
    }
}
