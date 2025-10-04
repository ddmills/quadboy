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
    DustyPlainsGroundLoot,
    CavernGroundLoot,
    MushroomForestGroundLoot,
    OpenAirGroundLoot,
    MountainGroundLoot,
    SwampGroundLoot,

    // Chest loot tables
    ForestChestLoot,
    DesertChestLoot,
    DustyPlainsChestLoot,
    CavernChestLoot,
    MushroomForestChestLoot,
    CommonChestLoot,
    MountainChestLoot,
    SwampChestLoot,

    // Enemy tables
    ForestEnemies,
    DesertEnemies,
    DustyPlainsEnemies,
    CavernEnemies,
    MushroomForestEnemies,
    OpenAirEnemies,
    MountainEnemies,
    SwampEnemies,

    // Death loot tables
    BanditLoot,
    BrownBearLoot,
    RattlesnakeLoot,
    BatLoot,
    CoyoteLoot,
    GiantFireflyLoot,
    BoulderLoot,
    RatLoot,
    BeetleLoot,
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
                .add(PrefabId::CanOfBeans, 3.5)
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
                .add(PrefabId::CanOfBeans, 3.0)
                .add(PrefabId::WoolShirt, 2.0)
                .add(PrefabId::Overcoat, 1.0)
                .add(PrefabId::SteelToeBoots, 1.0)
                .add(PrefabId::NavyRevolver, 0.5)
                .add(PrefabId::Amulet, 0.3)
                .add(PrefabId::Ring, 0.4)
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
                .add(PrefabId::CanOfBeans, 3.0)
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
                .add(PrefabId::CanOfBeans, 4.0)
                .add(PrefabId::Dynamite, 1.0)
                .add(PrefabId::SteelToeBoots, 1.0)
                .add(PrefabId::DoubleBarrelShotgun, 0.3)
                .add(PrefabId::NavyRevolver, 0.4)
                .add(PrefabId::Amulet, 0.2)
                .add(PrefabId::Ring, 0.3)
                .build(),
        );

        // Dusty Plains loot (mix of forest and desert items)
        self.tables.insert(
            LootTableId::DustyPlainsGroundLoot,
            LootTable::builder()
                .add(PrefabId::Lantern, 2.0)
                .add(PrefabId::Pickaxe, 2.0)
                .add(PrefabId::Campfire, 3.0)
                .add(PrefabId::Hatchet, 1.5)
                .add(PrefabId::Apple, 2.0)
                .add(PrefabId::CanOfBeans, 3.0)
                .add(PrefabId::Bedroll, 2.5)
                .build(),
        );

        self.tables.insert(
            LootTableId::DustyPlainsChestLoot,
            LootTable::builder()
                .add(PrefabId::Hatchet, 4.0)
                .add(PrefabId::Lantern, 3.5)
                .add(PrefabId::CavalrySword, 1.5)
                .add(PrefabId::Pickaxe, 3.0)
                .add(PrefabId::Apple, 2.0)
                .add(PrefabId::CanOfBeans, 3.5)
                .add(PrefabId::WoolShirt, 2.0)
                .add(PrefabId::Poncho, 2.0)
                .add(PrefabId::SteelToeBoots, 1.5)
                .add(PrefabId::NavyRevolver, 0.35)
                .add(PrefabId::Amulet, 0.25)
                .add(PrefabId::Ring, 0.25)
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
                .add(PrefabId::CanOfBeans, 2.5)
                .add(PrefabId::Dynamite, 3.0)
                .build(),
        );

        self.tables.insert(
            LootTableId::CavernChestLoot,
            LootTable::builder()
                .add(PrefabId::Pickaxe, 6.0)
                .add(PrefabId::Lantern, 5.0)
                .add(PrefabId::CavalrySword, 1.0)
                .add(PrefabId::CanOfBeans, 3.0)
                .add(PrefabId::Dynamite, 2.0)
                .add(PrefabId::Overcoat, 1.0)
                .add(PrefabId::SteelToeBoots, 2.0)
                .add(PrefabId::NavyRevolver, 0.3)
                .add(PrefabId::Amulet, 0.2)
                .add(PrefabId::Ring, 0.2)
                .build(),
        );

        // Mushroom Forest loot (similar to cavern but with more lanterns for lighting)
        self.tables.insert(
            LootTableId::MushroomForestGroundLoot,
            LootTable::builder()
                .add(PrefabId::Lantern, 2.0)
                .add(PrefabId::Pickaxe, 1.0)
                .add(PrefabId::Hatchet, 1.0)
                .add(PrefabId::Campfire, 2.0)
                .add(PrefabId::Apple, 1.5)
                .add(PrefabId::CanOfBeans, 2.0)
                .build(),
        );

        self.tables.insert(
            LootTableId::MushroomForestChestLoot,
            LootTable::builder()
                .add(PrefabId::Lantern, 6.0)
                .add(PrefabId::Pickaxe, 4.0)
                .add(PrefabId::CavalrySword, 1.0)
                .add(PrefabId::Apple, 3.0)
                .add(PrefabId::CanOfBeans, 3.5)
                .add(PrefabId::Hatchet, 2.0)
                .add(PrefabId::WoolShirt, 2.0)
                .add(PrefabId::Overcoat, 1.0)
                .add(PrefabId::SteelToeBoots, 1.5)
                .add(PrefabId::NavyRevolver, 0.3)
                .add(PrefabId::Amulet, 0.3)
                .add(PrefabId::Ring, 0.3)
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
                .add(PrefabId::Pickaxe, 5.0)
                .add(PrefabId::Campfire, 2.0)
                .add(PrefabId::Hatchet, 2.0)
                .add(PrefabId::CanOfBeans, 3.0)
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
                .add(PrefabId::CanOfBeans, 4.0)
                .add(PrefabId::WoolShirt, 4.0)
                .add(PrefabId::Overcoat, 3.0)
                .add(PrefabId::LongJohns, 3.0)
                .add(PrefabId::Dynamite, 2.0)
                .add(PrefabId::SteelToeBoots, 3.0)
                .add(PrefabId::LeverActionRifle, 0.2)
                .add(PrefabId::NavyRevolver, 0.3)
                .add(PrefabId::Amulet, 0.4)
                .add(PrefabId::Ring, 0.5)
                .build(),
        );

        // Enemy tables
        self.tables.insert(
            LootTableId::ForestEnemies,
            LootTable::builder()
                .add(PrefabId::Bandit, 1.0)
                .add(PrefabId::BrownBear, 0.2)
                .add(PrefabId::Coyote, 0.4)
                .add(PrefabId::GiantFirefly, 1.1)
                .build(),
        );

        self.tables.insert(
            LootTableId::DesertEnemies,
            LootTable::builder()
                .add(PrefabId::Bandit, 1.0)
                .add(PrefabId::Rattlesnake, 0.9)
                .add(PrefabId::Coyote, 0.5)
                .add(PrefabId::GiantBeetle, 0.8)
                .build(),
        );

        self.tables.insert(
            LootTableId::DustyPlainsEnemies,
            LootTable::builder()
                .add(PrefabId::Bandit, 1.0)
                .add(PrefabId::Coyote, 0.6)
                .add(PrefabId::Rattlesnake, 0.3)
                .add(PrefabId::GiantBeetle, 1.0)
                .build(),
        );

        self.tables.insert(
            LootTableId::CavernEnemies,
            LootTable::builder()
                .add(PrefabId::Bandit, 1.0)
                .add(PrefabId::Bat, 0.8)
                .add(PrefabId::Rat, 2.5)
                .build(),
        );

        self.tables.insert(
            LootTableId::MushroomForestEnemies,
            LootTable::builder()
                .add(PrefabId::Bandit, 0.5) // Fewer bandits in mushroom forests
                .add(PrefabId::Bat, 1.2) // More bats due to mushroom bioluminescence
                .add(PrefabId::Rat, 2.0)
                .add(PrefabId::GiantBeetle, 1.5) // Beetles love mushroom environments
                .build(),
        );

        self.tables
            .insert(LootTableId::OpenAirEnemies, LootTable::builder().build());

        self.tables.insert(
            LootTableId::MountainEnemies,
            LootTable::builder()
                .add(PrefabId::Bandit, 1.0)
                .add(PrefabId::BrownBear, 0.7)
                .add(PrefabId::Coyote, 0.6)
                .build(),
        );

        // Swamp loot
        self.tables.insert(
            LootTableId::SwampGroundLoot,
            LootTable::builder()
                .add(PrefabId::Lantern, 2.5)
                .add(PrefabId::Pickaxe, 2.0)
                .add(PrefabId::Campfire, 3.0)
                .add(PrefabId::Hatchet, 1.5)
                .add(PrefabId::Apple, 2.5)
                .add(PrefabId::CanOfBeans, 3.0)
                .add(PrefabId::Bedroll, 2.0)
                .build(),
        );

        self.tables.insert(
            LootTableId::SwampChestLoot,
            LootTable::builder()
                .add(PrefabId::Hatchet, 4.0)
                .add(PrefabId::Lantern, 5.0)
                .add(PrefabId::CavalrySword, 1.5)
                .add(PrefabId::Pickaxe, 2.5)
                .add(PrefabId::Apple, 2.5)
                .add(PrefabId::CanOfBeans, 3.5)
                .add(PrefabId::WoolShirt, 2.0)
                .add(PrefabId::Overcoat, 2.0)
                .add(PrefabId::SteelToeBoots, 1.0)
                .add(PrefabId::NavyRevolver, 0.4)
                .add(PrefabId::Amulet, 0.35)
                .add(PrefabId::Ring, 0.35)
                .build(),
        );

        self.tables.insert(
            LootTableId::SwampEnemies,
            LootTable::builder()
                .add(PrefabId::Bandit, 0.8)
                .add(PrefabId::Rattlesnake, 1.0)
                .add(PrefabId::Bat, 1.2)
                .add(PrefabId::GiantBeetle, 1.0)
                .add(PrefabId::Rat, 1.5)
                .build(),
        );

        // Common chest loot (fallback)
        self.tables.insert(
            LootTableId::CommonChestLoot,
            LootTable::builder()
                .add(PrefabId::Lantern, 1.0)
                .add(PrefabId::Pickaxe, 1.0)
                .add(PrefabId::Hatchet, 1.0)
                .add(PrefabId::Apple, 2.0)
                .add(PrefabId::CanOfBeans, 2.0)
                .add(PrefabId::Bedroll, 1.0)
                .add(PrefabId::Amulet, 0.1)
                .add(PrefabId::Ring, 0.1)
                .build(),
        );

        // Death loot tables
        self.tables.insert(
            LootTableId::BanditLoot,
            LootTable::builder()
                .add(PrefabId::GoldNugget, 1.0)
                .add(PrefabId::Amulet, 0.05)
                .add(PrefabId::Ring, 0.05)
                .build(),
        );

        self.tables.insert(
            LootTableId::BrownBearLoot,
            LootTable::builder()
                .add(PrefabId::GoldNugget, 0.5)
                .add(PrefabId::Apple, 1.0)
                .build(),
        );

        self.tables.insert(
            LootTableId::RattlesnakeLoot,
            LootTable::builder()
                .add(PrefabId::GoldNugget, 0.3)
                .add(PrefabId::Dynamite, 0.8)
                .build(),
        );

        self.tables.insert(
            LootTableId::BatLoot,
            LootTable::builder().add(PrefabId::GoldNugget, 0.2).build(),
        );
        self.tables.insert(
            LootTableId::RatLoot,
            LootTable::builder().add(PrefabId::GoldNugget, 0.1).build(),
        );

        self.tables.insert(
            LootTableId::CoyoteLoot,
            LootTable::builder()
                .add(PrefabId::GoldNugget, 0.4)
                .add(PrefabId::Apple, 0.8)
                .build(),
        );

        self.tables.insert(
            LootTableId::GiantFireflyLoot,
            LootTable::builder()
                .add(PrefabId::GoldNugget, 0.6)
                .add(PrefabId::Lantern, 0.5)
                .build(),
        );

        self.tables.insert(
            LootTableId::BoulderLoot,
            LootTable::builder().add(PrefabId::GoldNugget, 1.0).build(),
        );

        self.tables.insert(
            LootTableId::BeetleLoot,
            LootTable::builder()
                .add(PrefabId::GoldNugget, 0.5)
                .add(PrefabId::Pickaxe, 0.3)
                .build(),
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
