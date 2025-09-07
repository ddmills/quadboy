use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::SerializableComponent;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Item {
    pub weight: f32,
}

impl Item {
    pub fn new(weight: f32) -> Self {
        Self { weight }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Inventory {
    pub capacity: f32,
    pub item_ids: Vec<u64>,
    pub current_weight: f32,
}

impl Inventory {
    pub fn new(capacity: f32) -> Self {
        Self {
            capacity,
            item_ids: Vec::new(),
            current_weight: 0.0,
        }
    }

    pub fn add_item(&mut self, item_id: u64, weight: f32) -> bool {
        if self.current_weight + weight <= self.capacity {
            self.item_ids.push(item_id);
            self.current_weight += weight;
            true
        } else {
            false
        }
    }

    pub fn remove_item(&mut self, item_id: u64, weight: f32) -> bool {
        if let Some(index) = self.item_ids.iter().position(|&id| id == item_id) {
            self.item_ids.remove(index);
            self.current_weight -= weight;
            true
        } else {
            false
        }
    }

    pub fn count(&self) -> usize {
        self.item_ids.len()
    }

    pub fn contains_id(&self, item_id: u64) -> bool {
        self.item_ids.contains(&item_id)
    }

    pub fn has_space_for_weight(&self, weight: f32) -> bool {
        self.current_weight + weight <= self.capacity
    }

    pub fn get_total_weight(&self) -> f32 {
        self.current_weight
    }

    pub fn get_available_weight(&self) -> f32 {
        (self.capacity - self.current_weight).max(0.0)
    }
}

#[derive(Event)]
pub struct InventoryChangedEvent;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct InventoryAccessible;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct InInventory {
    pub owner_id: u64,
}

impl InInventory {
    pub fn new(owner_id: u64) -> Self {
        Self { owner_id }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct UnopenedContainer(pub crate::domain::LootTableId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StackableType {
    GoldNugget,
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Stackable {
    pub stack_type: StackableType,
}

impl Stackable {
    pub fn new(stack_type: StackableType) -> Self {
        Self { stack_type }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct StackCount {
    pub count: u32,
}

impl StackCount {
    pub const MAX_STACK_SIZE: u32 = 99;

    pub fn new(count: u32) -> Self {
        Self {
            count: count.min(Self::MAX_STACK_SIZE),
        }
    }

    pub fn add(&mut self, amount: u32) -> u32 {
        let space = Self::MAX_STACK_SIZE - self.count;
        let to_add = amount.min(space);
        self.count += to_add;
        amount - to_add // Return overflow
    }
}
