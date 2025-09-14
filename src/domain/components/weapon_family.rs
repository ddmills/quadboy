use serde::{Deserialize, Serialize};

use crate::domain::StatType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponFamily {
    Rifle,
    Shotgun,
    Pistol,
    Blade,
    Cudgel,
    Unarmed,
}

impl WeaponFamily {
    pub fn to_stat_type(&self) -> StatType {
        match self {
            WeaponFamily::Rifle => StatType::Rifle,
            WeaponFamily::Shotgun => StatType::Shotgun,
            WeaponFamily::Pistol => StatType::Pistol,
            WeaponFamily::Blade => StatType::Blade,
            WeaponFamily::Cudgel => StatType::Cudgel,
            WeaponFamily::Unarmed => StatType::Unarmed,
        }
    }
}