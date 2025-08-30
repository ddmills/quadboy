use super::{Prefab, Prefabs};
use bevy_ecs::{entity::Entity, world::World};

pub struct SpawnPrefabCommand {
    pub entity: Entity,
    pub config: Prefab,
}

impl SpawnPrefabCommand {
    pub fn new(entity: Entity, config: Prefab) -> Self {
        Self { entity, config }
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

        spawn_fn(self.entity, world, self.config);
        Ok(())
    }
}
