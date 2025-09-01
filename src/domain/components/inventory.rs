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
    pub capacity: usize,
    pub item_ids: Vec<u64>,
}

impl Inventory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            item_ids: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item_id: u64) -> bool {
        if self.item_ids.len() < self.capacity {
            self.item_ids.push(item_id);
            true
        } else {
            false
        }
    }

    pub fn remove_item(&mut self, item_id: u64) -> bool {
        if let Some(index) = self.item_ids.iter().position(|&id| id == item_id) {
            self.item_ids.remove(index);
            true
        } else {
            false
        }
    }

    pub fn is_full(&self) -> bool {
        self.item_ids.len() >= self.capacity
    }

    pub fn count(&self) -> usize {
        self.item_ids.len()
    }


    pub fn contains_id(&self, item_id: u64) -> bool {
        self.item_ids.contains(&item_id)
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct InInventory {
    pub owner_id: u64,
}

impl InInventory {
    pub fn new(owner_id: u64) -> Self {
        Self { owner_id }
    }
}
