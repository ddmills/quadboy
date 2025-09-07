#![allow(dead_code)]

use crate::common::Rand;

pub struct LootTable<T> {
    entries: Vec<LootEntry<T>>,
    total_weight: f32,
}

struct LootEntry<T> {
    item: T,
    weight: f32,
}

impl<T> LootTable<T> {
    pub fn builder() -> LootTableBuilder<T> {
        LootTableBuilder::new()
    }
}

impl<T: Clone> LootTable<T> {
    pub fn pick(&self, rand: &mut Rand) -> &T {
        if self.entries.is_empty() {
            panic!("Cannot pick from empty loot table");
        }

        if self.total_weight <= 0.0 {
            panic!("Cannot pick from loot table with zero total weight");
        }

        let mut target = rand.random() * self.total_weight;

        for entry in &self.entries {
            target -= entry.weight;
            if target <= 0.0 {
                return &entry.item;
            }
        }

        // Fallback to last entry if floating point precision issues
        &self.entries.last().unwrap().item
    }

    pub fn pick_cloned(&self, rand: &mut Rand) -> T {
        self.pick(rand).clone()
    }

    pub fn pick_multiple(&self, rand: &mut Rand, count: usize) -> Vec<T> {
        (0..count).map(|_| self.pick_cloned(rand)).collect()
    }

    /// Pick an item, guaranteed to return something (no Option wrapper)
    /// Use this when the loot table contains no None values
    pub fn pick_guaranteed(&self, rand: &mut Rand) -> &T {
        self.pick(rand)
    }

    /// Pick an item, guaranteed to return something (cloned)
    /// Use this when the loot table contains no None values  
    pub fn pick_guaranteed_cloned(&self, rand: &mut Rand) -> T {
        self.pick_cloned(rand)
    }

    /// Check if the loot table is empty (has no entries)
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub struct LootTableBuilder<T> {
    entries: Vec<LootEntry<T>>,
    total_weight: f32,
}

impl<T> LootTableBuilder<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            total_weight: 0.0,
        }
    }

    pub fn add(mut self, item: T, weight: f32) -> Self {
        if weight < 0.0 {
            panic!("Weight cannot be negative");
        }

        self.entries.push(LootEntry { item, weight });
        self.total_weight += weight;
        self
    }

    pub fn build(self) -> LootTable<T> {
        LootTable {
            entries: self.entries,
            total_weight: self.total_weight,
        }
    }
}

impl<T> Default for LootTableBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loot_table_basic() {
        let table = LootTable::builder()
            .add("common", 70.0)
            .add("rare", 20.0)
            .add("epic", 10.0)
            .build();

        let mut rand = Rand::seed(42);
        let result = table.pick(&mut rand);

        // Should pick something
        assert!(matches!(*result, "common" | "rare" | "epic"));
    }

    #[test]
    fn test_loot_table_multiple_picks() {
        let table = LootTable::builder().add(1, 50.0).add(2, 50.0).build();

        let mut rand = Rand::seed(42);
        let results = table.pick_multiple(&mut rand, 100);

        assert_eq!(results.len(), 100);

        // Should have both values represented
        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }

    #[test]
    #[should_panic(expected = "Cannot pick from empty loot table")]
    fn test_empty_table_panics() {
        let table: LootTable<i32> = LootTable {
            entries: Vec::new(),
            total_weight: 0.0,
        };
        let mut rand = Rand::seed(42);
        table.pick(&mut rand);
    }

    #[test]
    #[should_panic(expected = "Weight cannot be negative")]
    fn test_negative_weight_panics() {
        LootTable::builder().add("item", -1.0).build();
    }
}
