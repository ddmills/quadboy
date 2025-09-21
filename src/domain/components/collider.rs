use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct ColliderFlags: u32 {
        const BLOCKS_WALK       = 0b00000001;  // Blocks normal walking
        const BLOCKS_FLY        = 0b00000010;  // Blocks flying movement
        const BLOCKS_SWIM       = 0b00000100;  // Blocks swimming movement
        const BLOCKS_SIGHT      = 0b00100000;  // Blocks line of sight
        const BLOCKS_PROJECTILE = 0b01000000;  // Blocks projectiles
        const IS_ACTOR          = 0b10000000;  // Is an actor (can be attacked)

        // Common combinations
        const SOLID = Self::BLOCKS_WALK.bits() | Self::BLOCKS_SWIM.bits();
        const WALL = Self::SOLID.bits() | Self::BLOCKS_FLY.bits() | Self::BLOCKS_SIGHT.bits();
        const WATER = Self::BLOCKS_WALK.bits();
    }
}

bitflags! {
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct MovementFlags: u32 {
        const CAN_WALK   = 0b00000001;
        const CAN_FLY    = 0b00000010;
        const CAN_SWIM   = 0b00000100;

        // Common combinations
        const TERRESTRIAL = Self::CAN_WALK.bits();
        const AQUATIC = Self::CAN_SWIM.bits();
        const AMPHIBIOUS = Self::CAN_WALK.bits() | Self::CAN_SWIM.bits();
        const FLYING = Self::CAN_WALK.bits() | Self::CAN_FLY.bits();
    }
}

impl MovementFlags {
    pub fn is_blocked_by(self, collider_flags: ColliderFlags) -> bool {
        // Check if we have any way to move through this tile
        let can_walk_through = self.contains(MovementFlags::CAN_WALK)
            && !collider_flags.contains(ColliderFlags::BLOCKS_WALK);

        let can_fly_through = self.contains(MovementFlags::CAN_FLY)
            && !collider_flags.contains(ColliderFlags::BLOCKS_FLY);

        let can_swim_through = self.contains(MovementFlags::CAN_SWIM)
            && !collider_flags.contains(ColliderFlags::BLOCKS_SWIM);

        // We're blocked only if we have NO way through
        !(can_walk_through || can_fly_through || can_swim_through)
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Collider {
    pub flags: ColliderFlags,
}

impl Collider {
    pub fn new(flags: ColliderFlags) -> Self {
        Self { flags }
    }

    pub fn solid() -> Self {
        Self::new(ColliderFlags::SOLID)
    }

    pub fn wall() -> Self {
        Self::new(ColliderFlags::WALL)
    }

    pub fn water() -> Self {
        Self::new(ColliderFlags::WATER)
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct MovementCapabilities {
    pub flags: MovementFlags,
}

impl MovementCapabilities {
    pub fn new(flags: MovementFlags) -> Self {
        Self { flags }
    }

    pub fn terrestrial() -> Self {
        Self::new(MovementFlags::TERRESTRIAL)
    }

    pub fn aquatic() -> Self {
        Self::new(MovementFlags::AQUATIC)
    }

    pub fn amphibious() -> Self {
        Self::new(MovementFlags::AMPHIBIOUS)
    }

    pub fn flying() -> Self {
        Self::new(MovementFlags::FLYING)
    }
}
