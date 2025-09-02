use super::destruction_system::EntityDestroyedEvent;
use crate::{
    common::Rand,
    domain::{LootDrop, LootTableRegistry, Prefab, Prefabs},
};
use bevy_ecs::prelude::*;

pub fn on_entity_destroyed_loot(
    mut e_destroyed: EventReader<EntityDestroyedEvent>,
    q_loot_drops: Query<&LootDrop>,
    loot_registry: Res<LootTableRegistry>,
    mut rand: ResMut<Rand>,
    mut cmds: Commands,
) {
    for event in e_destroyed.read() {
        let Ok(loot_drop) = q_loot_drops.get(event.entity) else {
            continue;
        };

        if rand.bool(loot_drop.drop_chance) {
            let items =
                loot_registry.roll_multiple(loot_drop.loot_table, loot_drop.drop_count, &mut rand);

            for item_id in items {
                let config = Prefab::new(item_id, event.position);
                Prefabs::spawn(&mut cmds, config);
            }
        }
    }
}
