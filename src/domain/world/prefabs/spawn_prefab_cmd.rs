use crate::{
    domain::{NeedsStableId, PickupItemAction},
    engine::{StableId, StableIdRegistry},
};

use super::{Prefab, Prefabs};
use bevy_ecs::{entity::Entity, system::Command, world::World};

pub struct SpawnPrefabCommand {
    pub entity: Entity,
    pub config: Prefab,
    pub container_entity: Option<Entity>,
}

impl SpawnPrefabCommand {
    pub fn new(entity: Entity, config: Prefab) -> Self {
        Self {
            entity,
            config,
            container_entity: None,
        }
    }

    pub fn with_container(mut self, container_entity: Entity) -> Self {
        self.container_entity = Some(container_entity);
        self
    }

    pub fn execute(self, world: &mut World) -> Result<(), String> {
        let spawn_fn = {
            let prefabs = world
                .get_resource::<Prefabs>()
                .ok_or("Prefabs resource not found")?;

            *prefabs
                .spawn_functions
                .get(&self.config.prefab_id)
                .ok_or_else(|| format!("Unknown prefab type: {:?}", self.config.prefab_id))?
        };

        let builder = spawn_fn(self.entity, world, self.config);

        let builder = if let Some(_container) = self.container_entity {
            // Use for_container to prevent Position/StaticEntity and event firing
            builder.for_container()
        } else {
            builder
        };

        builder.build(self.entity, world);

        if let Some(container) = self.container_entity {
            let item_stable_id = {
                let mut stable_id_registry = world.resource_mut::<StableIdRegistry>();
                let id = stable_id_registry.generate_id();
                stable_id_registry.register(self.entity, id);
                id
            };

            world
                .entity_mut(self.entity)
                .insert(item_stable_id)
                .remove::<NeedsStableId>();

            PickupItemAction {
                entity: container,
                item_stable_id,
                spend_energy: false,
            }
            .apply(world);
        }

        Ok(())
    }
}
