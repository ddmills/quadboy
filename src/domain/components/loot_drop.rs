use crate::{domain::LootTableId, engine::SerializableComponent};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct LootDrop {
    pub loot_table: LootTableId,
    pub drop_chance: f32,
    pub drop_count: usize,
}

impl LootDrop {
    pub fn new(loot_table: LootTableId, drop_chance: f32) -> Self {
        Self {
            loot_table,
            drop_chance: drop_chance.clamp(0.0, 1.0),
            drop_count: 1,
        }
    }
}
