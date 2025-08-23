use super::{PrefabId, Prefabs, SpawnConfig};
use bevy_ecs::{entity::Entity, world::World};

pub struct SpawnPrefabCommand {
    pub entity: Entity,
    pub prefab_id: PrefabId,
    pub config: SpawnConfig,
}

impl SpawnPrefabCommand {
    pub fn new(entity: Entity, prefab_id: PrefabId, config: SpawnConfig) -> Self {
        Self {
            entity,
            prefab_id,
            config,
        }
    }

    pub fn execute(self, world: &mut World) -> Result<(), String> {
        let spawn_fn = {
            let prefabs = world
                .get_resource::<Prefabs>()
                .ok_or("Prefabs resource not found")?;

            *prefabs
                .spawn_functions
                .get(&self.prefab_id)
                .ok_or_else(|| format!("Unknown prefab type: {:?}", self.prefab_id))?
        };

        spawn_fn(self.entity, world, self.config);
        Ok(())
    }
}
