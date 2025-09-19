use crate::{
    common::Rand,
    domain::{
        GeneratedWeapon, ItemRarity, Weapon,
        world::prefabs::{Prefab, SpawnValue},
    },
};

/// Check if a prefab has rarity metadata and extract it
pub fn get_prefab_rarity(config: &Prefab) -> Option<ItemRarity> {
    if let Some(SpawnValue::ItemRarity(rarity)) = config.metadata.get("rarity") {
        Some(rarity.clone())
    } else {
        None
    }
}

/// Generate a weapon with rarity-based modifiers
pub fn generate_weapon_with_rarity(
    base_weapon: Weapon,
    base_name: &str,
    base_description: &str,
    rarity: ItemRarity,
) -> GeneratedWeapon {
    let mut rand = Rand::new(); // Use default seeded random for consistency

    // Create a GeneratedWeapon with the specified rarity
    let mut generated = GeneratedWeapon::new(base_weapon, base_name, base_description);
    generated.rarity = rarity;

    // Generate modifiers based on the rarity
    generated.generate_modifiers(&mut rand);

    // Apply modifiers to weapon stats
    generated.apply_modifiers();

    // Generate final name and description
    generated.generate_name_and_description(&mut rand);

    generated
}

/// Helper to generate a weapon from a prefab config
pub fn generate_weapon_from_prefab(
    config: &Prefab,
    base_weapon: Weapon,
    base_name: &str,
    base_description: &str,
) -> GeneratedWeapon {
    if let Some(rarity) = get_prefab_rarity(config) {
        // Use specified rarity
        generate_weapon_with_rarity(base_weapon, base_name, base_description, rarity)
    } else {
        // No rarity specified, roll for random rarity
        let mut rand = Rand::new();
        let random_rarity = ItemRarity::roll_random(&mut rand);
        generate_weapon_with_rarity(base_weapon, base_name, base_description, random_rarity)
    }
}
