use super::{
    SpawnPrefabCommand, spawn_bandit, spawn_boulder, spawn_pine_tree, spawn_stair_down,
    spawn_stair_up,
};
use crate::common::Palette;
use bevy_ecs::{entity::Entity, prelude::Resource, system::Commands, world::World};
use std::collections::HashMap;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum PrefabId {
    PineTree,
    Boulder,
    Bandit,
    StairDown,
    StairUp,
}

#[derive(Clone, Debug)]
pub struct SpawnConfig {
    pub pos: (usize, usize, usize),
    pub metadata: HashMap<String, SpawnValue>,
}

#[derive(Clone, Debug)]
pub enum SpawnValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}

impl SpawnConfig {
    pub fn new(pos: (usize, usize, usize)) -> Self {
        Self {
            pos,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: SpawnValue) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

type SpawnFunction = fn(Entity, &mut World, SpawnConfig);

#[derive(Resource)]
pub struct Prefabs {
    pub spawn_functions: HashMap<PrefabId, SpawnFunction>,
}

impl Prefabs {
    pub fn new() -> Self {
        let mut system = Self {
            spawn_functions: HashMap::new(),
        };

        system.register_all_prefabs();
        system
    }

    fn register_all_prefabs(&mut self) {
        self.register(PrefabId::PineTree, spawn_pine_tree);
        self.register(PrefabId::Boulder, spawn_boulder);
        self.register(PrefabId::Bandit, spawn_bandit);
        self.register(PrefabId::StairDown, spawn_stair_down);
        self.register(PrefabId::StairUp, spawn_stair_up);
    }

    pub fn register(&mut self, id: PrefabId, spawn_fn: SpawnFunction) {
        self.spawn_functions.insert(id, spawn_fn);
    }

    pub fn spawn(&self, cmds: &mut Commands, prefab_id: PrefabId, config: SpawnConfig) -> Entity {
        let entity = cmds.spawn_empty().id();

        let command = SpawnPrefabCommand::new(entity, prefab_id, config);
        cmds.queue(move |world: &mut World| {
            if let Err(e) = command.execute(world) {
                eprintln!("Failed to spawn prefab: {}", e);
            }
        });

        entity
    }

    pub fn spawn_world(world: &mut World, prefab_id: PrefabId, config: SpawnConfig) -> Entity {
        let entity = world.spawn_empty().id();

        let command = SpawnPrefabCommand::new(entity, prefab_id, config);
        let _ = command.execute(world);

        entity
    }
}

impl Default for Prefabs {
    fn default() -> Self {
        Self::new()
    }
}
