use super::{
    SpawnPrefabCommand, spawn_apple, spawn_bandit, spawn_bedroll, spawn_boulder, spawn_cactus,
    spawn_campfire, spawn_cavalry_sword, spawn_chest, spawn_duster, spawn_dynamite,
    spawn_giant_mushroom, spawn_hatchet, spawn_lantern, spawn_long_johns, spawn_overcoat,
    spawn_pickaxe, spawn_pine_tree, spawn_poncho, spawn_stair_down, spawn_stair_up,
    spawn_steel_toe_boots, spawn_terrain_tile, spawn_wool_shirt,
    spawn_lever_action_rifle, spawn_double_barrel_shotgun, spawn_navy_revolver,
};
use crate::domain::{LootTableId, Terrain, spawn_gold_nugget};
use bevy_ecs::{entity::Entity, prelude::Resource, system::Commands, world::World};
use std::{collections::HashMap, fmt};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum PrefabId {
    PineTree,
    Boulder,
    Campfire,
    GoldNugget,
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
    Dynamite,
    Apple,
    Bedroll,
    LongJohns,
    Duster,
    Poncho,
    Overcoat,
    WoolShirt,
    SteelToeBoots,
    LeverActionRifle,
    DoubleBarrelShotgun,
    NavyRevolver,
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
        self.register(PrefabId::Campfire, spawn_campfire);
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
        self.register(PrefabId::GoldNugget, spawn_gold_nugget);
        self.register(PrefabId::Dynamite, spawn_dynamite);
        self.register(PrefabId::Apple, spawn_apple);
        self.register(PrefabId::Bedroll, spawn_bedroll);
        self.register(PrefabId::LongJohns, spawn_long_johns);
        self.register(PrefabId::Duster, spawn_duster);
        self.register(PrefabId::Poncho, spawn_poncho);
        self.register(PrefabId::Overcoat, spawn_overcoat);
        self.register(PrefabId::WoolShirt, spawn_wool_shirt);
        self.register(PrefabId::SteelToeBoots, spawn_steel_toe_boots);
        self.register(PrefabId::LeverActionRifle, spawn_lever_action_rifle);
        self.register(PrefabId::DoubleBarrelShotgun, spawn_double_barrel_shotgun);
        self.register(PrefabId::NavyRevolver, spawn_navy_revolver);

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

impl fmt::Display for PrefabId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrefabId::PineTree => write!(f, "Pine Tree"),
            PrefabId::Boulder => write!(f, "Boulder"),
            PrefabId::Campfire => write!(f, "Campfire"),
            PrefabId::GoldNugget => write!(f, "Gold Nugget"),
            PrefabId::Cactus => write!(f, "Cactus"),
            PrefabId::CavalrySword => write!(f, "Cavalry Sword"),
            PrefabId::Chest => write!(f, "Chest"),
            PrefabId::GiantMushroom => write!(f, "Giant Mushroom"),
            PrefabId::Bandit => write!(f, "Bandit"),
            PrefabId::Hatchet => write!(f, "Hatchet"),
            PrefabId::Lantern => write!(f, "Lantern"),
            PrefabId::Pickaxe => write!(f, "Pickaxe"),
            PrefabId::StairDown => write!(f, "Stair Down"),
            PrefabId::StairUp => write!(f, "Stair Up"),
            PrefabId::Dynamite => write!(f, "Dynamite"),
            PrefabId::Apple => write!(f, "Apple"),
            PrefabId::Bedroll => write!(f, "Bedroll"),
            PrefabId::LongJohns => write!(f, "Long Johns"),
            PrefabId::Duster => write!(f, "Duster"),
            PrefabId::Poncho => write!(f, "Poncho"),
            PrefabId::Overcoat => write!(f, "Overcoat"),
            PrefabId::WoolShirt => write!(f, "Wool Shirt"),
            PrefabId::SteelToeBoots => write!(f, "Steel Toe Boots"),
            PrefabId::LeverActionRifle => write!(f, "Lever Action Rifle"),
            PrefabId::DoubleBarrelShotgun => write!(f, "Double Barrel Shotgun"),
            PrefabId::NavyRevolver => write!(f, "Navy Revolver"),
            PrefabId::TerrainTile(terrain) => match terrain {
                Terrain::Grass => write!(f, "Grass Tile"),
                Terrain::Dirt => write!(f, "Dirt Tile"),
                Terrain::River => write!(f, "River Tile"),
                Terrain::Sand => write!(f, "Sand Tile"),
                Terrain::Shallows => write!(f, "Shallows Tile"),
                Terrain::OpenAir => write!(f, "Open Air Tile"),
            },
        }
    }
}
