use std::collections::HashMap;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::SerializableComponent;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    Head,
    Body,
    Legs,
    Feet,
    MainHand,
    OffHand,
    BothHands, // For two-handed items
    Ring1,
    Ring2,
    Neck,
}

impl EquipmentSlot {
    pub fn display_name(&self) -> &'static str {
        match self {
            EquipmentSlot::Head => "Head",
            EquipmentSlot::Body => "Body",
            EquipmentSlot::Legs => "Legs",
            EquipmentSlot::Feet => "Feet",
            EquipmentSlot::MainHand => "Main Hand",
            EquipmentSlot::OffHand => "Off Hand",
            EquipmentSlot::BothHands => "Both Hands",
            EquipmentSlot::Ring1 => "Ring 1",
            EquipmentSlot::Ring2 => "Ring 2",
            EquipmentSlot::Neck => "Neck",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipmentType {
    Weapon,
    Armor,
    Accessory,
    Tool,
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct EquipmentSlots {
    pub slots: HashMap<EquipmentSlot, Option<u64>>,
}

impl EquipmentSlots {
    pub fn humanoid() -> Self {
        let mut slots = HashMap::new();
        slots.insert(EquipmentSlot::Head, None);
        slots.insert(EquipmentSlot::Body, None);
        slots.insert(EquipmentSlot::Legs, None);
        slots.insert(EquipmentSlot::Feet, None);
        slots.insert(EquipmentSlot::MainHand, None);
        slots.insert(EquipmentSlot::OffHand, None);
        slots.insert(EquipmentSlot::Ring1, None);
        slots.insert(EquipmentSlot::Ring2, None);
        slots.insert(EquipmentSlot::Neck, None);
        Self { slots }
    }

    pub fn can_equip(&self, item: &Equippable) -> bool {
        // Check if all required slots are free
        for slot in &item.slot_requirements {
            match slot {
                EquipmentSlot::BothHands => {
                    // Check both main and off hand are free
                    if self
                        .slots
                        .get(&EquipmentSlot::MainHand)
                        .unwrap_or(&None)
                        .is_some()
                        || self
                            .slots
                            .get(&EquipmentSlot::OffHand)
                            .unwrap_or(&None)
                            .is_some()
                    {
                        return false;
                    }
                }
                _ => {
                    if self.slots.get(slot).unwrap_or(&None).is_some() {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn equip(&mut self, item_id: u64, slots: &[EquipmentSlot]) {
        for slot in slots {
            match slot {
                EquipmentSlot::BothHands => {
                    // Occupy both hand slots
                    self.slots.insert(EquipmentSlot::MainHand, Some(item_id));
                    self.slots.insert(EquipmentSlot::OffHand, Some(item_id));
                }
                _ => {
                    self.slots.insert(*slot, Some(item_id));
                }
            }
        }
    }

    pub fn unequip(&mut self, slot: EquipmentSlot) -> Option<u64> {
        // Get the item ID from the slot
        let item_id = self.slots.get(&slot).and_then(|&id| id)?;

        // Clear the slot
        self.slots.insert(slot, None);

        // If it's a two-handed item, clear both slots
        if slot == EquipmentSlot::MainHand || slot == EquipmentSlot::OffHand {
            // Check if the other hand has the same item (two-handed)
            let other_slot = if slot == EquipmentSlot::MainHand {
                EquipmentSlot::OffHand
            } else {
                EquipmentSlot::MainHand
            };

            if self.slots.get(&other_slot) == Some(&Some(item_id)) {
                self.slots.insert(other_slot, None);
            }
        }

        Some(item_id)
    }

    pub fn get_equipped_item(&self, slot: EquipmentSlot) -> Option<u64> {
        self.slots.get(&slot).and_then(|&id| id)
    }

    pub fn get_all_equipped(&self) -> Vec<u64> {
        let mut items = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for &item_id in self.slots.values() {
            if let Some(id) = item_id {
                if seen.insert(id) {
                    items.push(id);
                }
            }
        }

        items
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Equippable {
    pub slot_requirements: Vec<EquipmentSlot>,
    pub equipment_type: EquipmentType,
}

impl Equippable {
    pub fn new(slots: Vec<EquipmentSlot>, eq_type: EquipmentType) -> Self {
        Self {
            slot_requirements: slots,
            equipment_type: eq_type,
        }
    }

    pub fn weapon_one_handed() -> Self {
        Self::new(vec![EquipmentSlot::MainHand], EquipmentType::Weapon)
    }

    pub fn weapon_two_handed() -> Self {
        Self::new(vec![EquipmentSlot::BothHands], EquipmentType::Weapon)
    }

    pub fn tool() -> Self {
        Self::new(vec![EquipmentSlot::MainHand], EquipmentType::Tool)
    }

    pub fn armor_head() -> Self {
        Self::new(vec![EquipmentSlot::Head], EquipmentType::Armor)
    }

    pub fn armor_body() -> Self {
        Self::new(vec![EquipmentSlot::Body], EquipmentType::Armor)
    }

    pub fn armor_legs() -> Self {
        Self::new(vec![EquipmentSlot::Legs], EquipmentType::Armor)
    }

    pub fn armor_feet() -> Self {
        Self::new(vec![EquipmentSlot::Feet], EquipmentType::Armor)
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Equipped {
    pub owner_id: u64,
    pub slots: Vec<EquipmentSlot>,
}

impl Equipped {
    pub fn new(owner_id: u64, slots: Vec<EquipmentSlot>) -> Self {
        Self { owner_id, slots }
    }
}
