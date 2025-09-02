use super::{
    SpawnPrefabCommand, spawn_bandit, spawn_boulder, spawn_cactus, spawn_cavalry_sword,
    spawn_chest, spawn_giant_mushroom, spawn_hatchet, spawn_lantern, spawn_pickaxe,
    spawn_pine_tree, spawn_stair_down, spawn_stair_up, spawn_terrain_tile,
};
use crate::{
    domain::{LootTableId, PickupItemAction, Terrain},
    engine::StableIdRegistry,
};
use bevy_ecs::{entity::Entity, prelude::Resource, system::Commands, world::World};
use std::collections::HashMap;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum PrefabId {
    PineTree,
    Boulder,
    Cactus,
    CavalrySword,
    Chest,
    GiantMushroom,
    Bandit,
    Hatchet,
    Lantern,
    Pickaxe,
    StairDown,
    StairUp,
    TerrainTile(Terrain),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Prefab {
    pub prefab_id: PrefabId,
    pub pos: (usize, usize, usize),
    pub metadata: HashMap<String, SpawnValue>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum SpawnValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    LootTableId(LootTableId),
}

impl Prefab {
    pub fn new(prefab_id: PrefabId, pos: (usize, usize, usize)) -> Self {
        Self {
            prefab_id,
            pos,
            metadata: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_metadata(mut self, key: String, value: SpawnValue) -> Self {
        self.metadata.insert(key, value);
        self
    }

    #[allow(dead_code)]
    pub fn with_pos(mut self, pos: (usize, usize, usize)) -> Self {
        self.pos = pos;
        self
    }
}

type SpawnFunction = fn(Entity, &mut World, Prefab);

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
        self.register(PrefabId::Cactus, spawn_cactus);
        self.register(PrefabId::CavalrySword, spawn_cavalry_sword);
        self.register(PrefabId::Chest, spawn_chest);
        self.register(PrefabId::GiantMushroom, spawn_giant_mushroom);
        self.register(PrefabId::Bandit, spawn_bandit);
        self.register(PrefabId::Hatchet, spawn_hatchet);
        self.register(PrefabId::Lantern, spawn_lantern);
        self.register(PrefabId::Pickaxe, spawn_pickaxe);
        self.register(PrefabId::StairDown, spawn_stair_down);
        self.register(PrefabId::StairUp, spawn_stair_up);

        self.register(PrefabId::TerrainTile(Terrain::Grass), spawn_terrain_tile);
        self.register(PrefabId::TerrainTile(Terrain::Dirt), spawn_terrain_tile);
        self.register(PrefabId::TerrainTile(Terrain::River), spawn_terrain_tile);
        self.register(PrefabId::TerrainTile(Terrain::Sand), spawn_terrain_tile);
        self.register(PrefabId::TerrainTile(Terrain::Shallows), spawn_terrain_tile);
    }

    pub fn register(&mut self, id: PrefabId, spawn_fn: SpawnFunction) {
        self.spawn_functions.insert(id, spawn_fn);
    }

    pub fn spawn(cmds: &mut Commands, config: Prefab) -> Entity {
        let entity = cmds.spawn_empty().id();

        let command = SpawnPrefabCommand::new(entity, config);
        cmds.queue(move |world: &mut World| {
            if let Err(e) = command.execute(world) {
                eprintln!("Failed to spawn prefab: {}", e);
            }
        });

        entity
    }

    pub fn spawn_world(world: &mut World, config: Prefab) -> Entity {
        let entity = world.spawn_empty().id();

        let command = SpawnPrefabCommand::new(entity, config);
        let _ = command.execute(world);

        entity
    }

    pub fn spawn_in_container(
        world: &mut World,
        config: Prefab,
        container_entity: Entity,
    ) -> Entity {
        let entity = world.spawn_empty().id();

        let command = SpawnPrefabCommand::new(entity, config).with_container(container_entity);

        let _ = command.execute(world);

        entity
    }
}

impl Default for Prefabs {
    fn default() -> Self {
        Self::new()
    }
}
