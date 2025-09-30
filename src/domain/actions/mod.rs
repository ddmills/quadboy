use bevy_ecs::prelude::*;

pub trait GameAction {
    fn try_apply(self, world: &mut World) -> bool;
}

mod attack_action;
mod drop_item_action;
mod eat_action;
mod equip_item_action;
mod move_action;
mod open_container_action;
mod pickup_item_action;
mod reload_action;
mod stack_split_util;
mod throw_item_action;
mod toggle_light_action;
mod transfer_item_action;
mod unequip_item_action;
mod wait_action;

pub use attack_action::*;
pub use drop_item_action::*;
pub use eat_action::*;
pub use equip_item_action::*;
pub use move_action::*;
pub use open_container_action::*;
pub use pickup_item_action::*;
pub use reload_action::*;
pub use stack_split_util::*;
pub use throw_item_action::*;
pub use toggle_light_action::*;
pub use transfer_item_action::*;
pub use unequip_item_action::*;
pub use wait_action::*;
