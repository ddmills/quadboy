use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponFamily {
    Rifle,
    Shotgun,
    Pistol,
    Blade,
    Cudgel,
    Unarmed,
}