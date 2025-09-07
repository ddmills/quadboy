use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::{
    common::{LootTable, Rand},
    domain::PrefabId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum LootTableId {
    // Ground loot tables
    ForestGroundLoot,
    DesertGroundLoot,
    CavernGroundLoot,
    OpenAirGroundLoot,
    MountainGroundLoot,

    // Chest loot tables
    ForestChestLoot,
    DesertChestLoot,
    CavernChestLoot,
    CommonChestLoot,
    MountainChestLoot,

    // Enemy tables
    ForestEnemies,
    DesertEnemies,
    CavernEnemies,
    OpenAirEnemies,
    MountainEnemies,

    // Death loot tables
    BanditLoot,
    BoulderLoot,
}

#[derive(Resource)]
pub struct LootTableRegistry {
    tables: HashMap<LootTableId, LootTable<PrefabId>>,
}

impl LootTableRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tables: HashMap::new(),
        };
        registry.initialize_tables();
        registry
    }

    fn initialize_tables(&mut self) {
        // Forest loot
        self.tables.insert(
            LootTableId::ForestGroundLoot,
            LootTable::builder()
                .add(PrefabId::Lantern, 3.0)
                .add(PrefabId::Pickaxe, 3.0)
                .add(PrefabId::Campfire, 3.0)
                .add(PrefabId::Hatchet, 1.0)
                .add(PrefabId::Apple, 4.0)
                .add(PrefabId::Bedroll, 2.0)
                .build(),
        );

        self.tables.insert(
            LootTableId::ForestChestLoot,
            LootTable::builder()
                .add(PrefabId::Hatchet, 5.0)
                .add(PrefabId::Lantern, 3.0)
                .add(PrefabId::CavalrySword, 1.0)
                .add(PrefabId::Pickaxe, 2.0)
                .add(PrefabId::Apple, 3.0)
                .add(PrefabId::WoolShirt, 2.0)
                .add(PrefabId::Overcoat, 1.0)
                .add(PrefabId::SteelToeBoots, 1.0)
                .add(PrefabId::NavyRevolver, 0.5)
                .build(),
        );

        // Desert loot
        self.tables.insert(
            LootTableId::DesertGroundLoot,
            LootTable::builder()
                .add(PrefabId::Lantern, 1.0)
                .add(PrefabId::Pickaxe, 1.0)
                .add(PrefabId::Hatchet, 1.0)
                .add(PrefabId::Campfire, 3.0)
                .add(PrefabId::Dynamite, 2.0)
                .add(PrefabId::Bedroll, 2.0)
                .build(),
        );

        self.tables.insert(
            LootTableId::DesertChestLoot,
            LootTable::builder()
                .add(PrefabId::Pickaxe, 5.0)
                .add(PrefabId::Lantern, 4.0)
                .add(PrefabId::CavalrySword, 2.0)
                .add(PrefabId::Poncho, 3.0)
                .add(PrefabId::Duster, 2.0)
                .add(PrefabId::Dynamite, 1.0)
                .add(PrefabId::SteelToeBoots, 1.0)
                .add(PrefabId::DoubleBarrelShotgun, 0.3)
                .add(PrefabId::NavyRevolver, 0.4)
                .build(),
        );

        // Cavern loot
        self.tables.insert(
            LootTableId::CavernGroundLoot,
            LootTable::builder()
                .add(PrefabId::Lantern, 1.0)
                .add(PrefabId::Pickaxe, 1.0)
                .add(PrefabId::Hatchet, 1.0)
                .add(PrefabId::Campfire, 3.0)
                .add(PrefabId::Dynamite, 3.0)
                .build(),
        );

        self.tables.insert(
            LootTableId::CavernChestLoot,
            LootTable::builder()
                .add(PrefabId::Pickaxe, 6.0)
                .add(PrefabId::Lantern, 5.0)
                .add(PrefabId::CavalrySword, 1.0)
                .add(PrefabId::Dynamite, 2.0)
                .add(PrefabId::Overcoat, 1.0)
                .add(PrefabId::SteelToeBoots, 2.0)
                .add(PrefabId::NavyRevolver, 0.3)
                .build(),
        );

        // OpenAir (minimal loot)
        self.tables
            .insert(LootTableId::OpenAirGroundLoot, LootTable::builder().build());

        // Mountain loot
        self.tables.insert(
            LootTableId::MountainGroundLoot,
            LootTable::builder()
                .add(PrefabId::Lantern, 2.0)
                .add(PrefabId::Pickaxe, 5.0) // More mining tools in mountains
                .add(PrefabId::Campfire, 2.0)
                .add(PrefabId::Hatchet, 2.0)
                .add(PrefabId::Bedroll, 3.0)
                .add(PrefabId::Dynamite, 4.0)
                .build(),
        );

        self.tables.insert(
            LootTableId::MountainChestLoot,
            LootTable::builder()
                .add(PrefabId::Pickaxe, 6.0)
                .add(PrefabId::Hatchet, 4.0)
                .add(PrefabId::Lantern, 3.0)
                .add(PrefabId::CavalrySword, 1.0)
                .add(PrefabId::WoolShirt, 4.0)
                .add(PrefabId::Overcoat, 3.0)
                .add(PrefabId::LongJohns, 3.0)
                .add(PrefabId::Dynamite, 2.0)
                .add(PrefabId::SteelToeBoots, 3.0)
                .add(PrefabId::LeverActionRifle, 0.2)
                .add(PrefabId::NavyRevolver, 0.3)
                .build(),
        );

        // Enemy tables
        self.tables.insert(
            LootTableId::ForestEnemies,
            LootTable::builder().add(PrefabId::Bandit, 1.0).build(),
        );

        self.tables.insert(
            LootTableId::DesertEnemies,
            LootTable::builder().add(PrefabId::Bandit, 1.0).build(),
        );

        self.tables.insert(
            LootTableId::CavernEnemies,
            LootTable::builder().add(PrefabId::Bandit, 1.0).build(),
        );

        self.tables
            .insert(LootTableId::OpenAirEnemies, LootTable::builder().build());

        self.tables.insert(
            LootTableId::MountainEnemies,
            LootTable::builder().add(PrefabId::Bandit, 1.0).build(),
        );

        // Common chest loot (fallback)
        self.tables.insert(
            LootTableId::CommonChestLoot,
            LootTable::builder()
                .add(PrefabId::Lantern, 1.0)
                .add(PrefabId::Pickaxe, 1.0)
                .add(PrefabId::Hatchet, 1.0)
                .add(PrefabId::Apple, 2.0)
                .add(PrefabId::Bedroll, 1.0)
                .build(),
        );

        // Death loot tables
        self.tables.insert(
            LootTableId::BanditLoot,
            LootTable::builder().add(PrefabId::GoldNugget, 1.0).build(),
        );

        self.tables.insert(
            LootTableId::BoulderLoot,
            LootTable::builder().add(PrefabId::GoldNugget, 1.0).build(),
        );
    }

    pub fn get(&self, id: LootTableId) -> Option<&LootTable<PrefabId>> {
        self.tables.get(&id)
    }

    pub fn roll(&self, id: LootTableId, rand: &mut Rand) -> Option<PrefabId> {
        self.get(id).map(|table| table.pick_cloned(rand))
    }

    pub fn roll_guaranteed(&self, id: LootTableId, rand: &mut Rand) -> PrefabId {
        self.get(id)
            .map(|table| table.pick_guaranteed_cloned(rand))
            .unwrap_or(PrefabId::Lantern) // fallback
    }

    pub fn roll_multiple(&self, id: LootTableId, count: usize, rand: &mut Rand) -> Vec<PrefabId> {
        let mut items = Vec::new();
        if let Some(table) = self.get(id) {
            for _ in 0..count {
                let item = table.pick_guaranteed_cloned(rand);
                items.push(item);
            }
        }
        items
    }

    pub fn is_empty(&self, id: LootTableId) -> bool {
        self.get(id).is_none_or(|table| table.is_empty())
    }
}
