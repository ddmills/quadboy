use crate::{
    common::Rand,
    domain::{
        HitEffect, ItemRarity, Weapon, WeaponModifier, WeaponModifierType, pick_legendary_name,
        pick_random_prefix, pick_random_suffix,
    },
};

/// Represents a generated weapon with modifiers applied
#[derive(Debug, Clone)]
pub struct GeneratedWeapon {
    pub weapon: Weapon,
    pub name: String,
    pub description: String,
    pub rarity: ItemRarity,
    pub modifiers: Vec<WeaponModifier>,
}

impl GeneratedWeapon {
    /// Create a new generated weapon from a base weapon
    pub fn new(base_weapon: Weapon, base_name: &str, base_description: &str) -> Self {
        Self {
            weapon: base_weapon,
            name: base_name.to_string(),
            description: base_description.to_string(),
            rarity: ItemRarity::Common,
            modifiers: Vec::new(),
        }
    }

    /// Generate a random weapon with modifiers
    pub fn generate_random(
        base_weapon: Weapon,
        base_name: &str,
        base_description: &str,
        tier_bonus: f32, // 0.0 to 1.0 - increases chance of higher rarity
        rand: &mut Rand,
    ) -> Self {
        let mut generated = Self::new(base_weapon, base_name, base_description);

        // Roll for rarity with tier bonus
        generated.rarity = Self::roll_rarity_with_bonus(tier_bonus, rand);

        // Generate modifiers based on rarity
        generated.generate_modifiers(rand);

        // Apply modifiers to weapon stats
        generated.apply_modifiers();

        // Generate final name and description
        generated.generate_name_and_description(rand);

        generated
    }

    /// Roll for rarity with a tier bonus (higher bonus = better rarity chance)
    fn roll_rarity_with_bonus(tier_bonus: f32, rand: &mut Rand) -> ItemRarity {
        let roll = rand.random() + tier_bonus.clamp(0.0, 0.3); // Cap bonus at 30%

        if roll < 0.50 {
            ItemRarity::Common
        } else if roll < 0.75 {
            ItemRarity::Uncommon
        } else if roll < 0.90 {
            ItemRarity::Rare
        } else if roll < 0.98 {
            ItemRarity::Epic
        } else {
            ItemRarity::Legendary
        }
    }

    /// Generate modifiers based on rarity
    pub fn generate_modifiers(&mut self, rand: &mut Rand) {
        let min_modifiers = self.rarity.min_modifiers();
        let max_modifiers = self.rarity.max_modifiers();

        if max_modifiers == 0 {
            return;
        }

        // Determine number of modifiers to add
        let num_modifiers = if min_modifiers == max_modifiers {
            max_modifiers
        } else {
            let range = max_modifiers - min_modifiers + 1;
            min_modifiers + ((rand.random() * range as f32) as usize)
        };

        // Generate modifiers
        let mut has_prefix = false;
        let mut has_suffix = false;

        for _ in 0..num_modifiers {
            // Decide whether to add prefix or suffix
            let add_prefix = if !has_prefix && !has_suffix {
                // First modifier, 50/50 chance
                rand.random() < 0.5
            } else if !has_prefix {
                // Only prefix available
                true
            } else if !has_suffix {
                // Only suffix available
                false
            } else {
                // Both slots filled, skip
                break;
            };

            if add_prefix && !has_prefix {
                if let Some(prefix) = pick_random_prefix(self.weapon.weapon_type, rand) {
                    self.modifiers.push(prefix);
                    has_prefix = true;
                }
            } else if !add_prefix
                && !has_suffix
                && let Some(suffix) = pick_random_suffix(self.weapon.weapon_type, rand)
            {
                self.modifiers.push(suffix);
                has_suffix = true;
            }
        }
    }

    /// Apply all modifiers to the weapon
    pub fn apply_modifiers(&mut self) {
        for modifier in &self.modifiers {
            // Apply damage bonus
            if modifier.damage_bonus != 0 {
                self.weapon.damage_dice =
                    self.apply_damage_bonus(&self.weapon.damage_dice, modifier.damage_bonus);
            }

            // Apply energy cost modifier
            // Note: This would need to be stored somewhere on the weapon for the energy system to use

            // Apply range modifier
            if let Some(ref mut range) = self.weapon.range {
                *range = (*range as i32 + modifier.range_modifier).max(1) as usize;
            }

            // Add hit effects
            self.weapon.hit_effects.extend(modifier.hit_effects.clone());

            // Apply special damage bonuses (modify damage dice for specific materials)
            for &(material, bonus) in &modifier.special_damage {
                if self.weapon.can_damage.contains(&material) {
                    self.weapon.damage_dice =
                        self.apply_damage_bonus(&self.weapon.damage_dice, bonus);
                }
            }
        }
    }

    /// Apply a damage bonus to a dice string (e.g., "1d6+1" -> "1d6+3" for +2 bonus)
    fn apply_damage_bonus(&self, dice_string: &str, bonus: i32) -> String {
        // Parse existing dice string
        if let Some((base, existing_bonus)) = self.parse_dice_string(dice_string) {
            let new_bonus = existing_bonus + bonus;
            if new_bonus > 0 {
                format!("{}+{}", base, new_bonus)
            } else if new_bonus < 0 {
                format!("{}{}", base, new_bonus) // negative bonus already has minus sign
            } else {
                base
            }
        } else {
            // Fallback if parsing fails
            dice_string.to_string()
        }
    }

    /// Parse a dice string like "1d6+2" into ("1d6", 2)
    fn parse_dice_string(&self, dice_string: &str) -> Option<(String, i32)> {
        if let Some(plus_pos) = dice_string.find('+') {
            let base = dice_string[..plus_pos].to_string();
            let bonus_str = &dice_string[plus_pos + 1..];
            if let Ok(bonus) = bonus_str.parse::<i32>() {
                return Some((base, bonus));
            }
        } else if let Some(minus_pos) = dice_string.rfind('-')
            && minus_pos > 0
        {
            // Make sure it's not a negative die count
            let base = dice_string[..minus_pos].to_string();
            let bonus_str = &dice_string[minus_pos..]; // Include the minus sign
            if let Ok(bonus) = bonus_str.parse::<i32>() {
                return Some((base, bonus));
            }
        }

        // No bonus found, return base with 0 bonus
        Some((dice_string.to_string(), 0))
    }

    /// Generate the final name and description
    pub fn generate_name_and_description(&mut self, rand: &mut Rand) {
        // Generate name
        let mut name_parts = Vec::new();

        // Add rarity prefix for epic/legendary
        if matches!(self.rarity, ItemRarity::Epic | ItemRarity::Legendary) {
            name_parts.push(self.rarity.get_display_name().to_string());
        }

        // Add prefix modifier name
        if let Some(prefix) = self
            .modifiers
            .iter()
            .find(|m| matches!(m.modifier_type, WeaponModifierType::Prefix))
        {
            name_parts.push(prefix.name.clone());
        }

        // Add base weapon name
        name_parts.push(self.name.clone());

        // Add suffix modifier name
        if let Some(suffix) = self
            .modifiers
            .iter()
            .find(|m| matches!(m.modifier_type, WeaponModifierType::Suffix))
        {
            name_parts.push(suffix.name.clone());
        }

        // For legendary items, optionally use a special name
        if self.rarity == ItemRarity::Legendary {
            if let Some(legendary_name) = pick_legendary_name(self.weapon.weapon_type, rand) {
                // 50% chance to use legendary name instead
                if rand.random() < 0.5 {
                    self.name = format!("\"{}\"", legendary_name);
                } else {
                    self.name = name_parts.join(" ");
                }
            } else {
                self.name = name_parts.join(" ");
            }
        } else {
            self.name = name_parts.join(" ");
        }

        // Generate description
        self.generate_description();
    }

    /// Generate a description that includes modifier effects
    fn generate_description(&mut self) {
        let mut description_parts = vec![self.description.clone()];

        // Add modifier descriptions
        for modifier in &self.modifiers {
            if !modifier.description.is_empty() {
                description_parts.push(modifier.description.clone());
            }
        }

        // Add effect summaries
        if !self.weapon.hit_effects.is_empty() {
            let mut effect_descriptions = Vec::new();
            for effect in &self.weapon.hit_effects {
                match effect {
                    HitEffect::Knockback { chance, .. } => {
                        if *chance > 0.8 {
                            effect_descriptions.push("Powerful knockback on hit".to_string());
                        } else if *chance > 0.3 {
                            effect_descriptions.push("May knock back enemies".to_string());
                        }
                    }
                    HitEffect::Poison { chance, .. } => {
                        if *chance > 0.5 {
                            effect_descriptions.push("Inflicts deadly poison".to_string());
                        } else {
                            effect_descriptions.push("May poison enemies".to_string());
                        }
                    }
                    HitEffect::Bleeding { chance, .. } => {
                        if *chance > 0.5 {
                            effect_descriptions.push("Causes severe bleeding".to_string());
                        } else {
                            effect_descriptions.push("May cause bleeding wounds".to_string());
                        }
                    }
                    HitEffect::Burning { chance, .. } => {
                        if *chance > 0.5 {
                            effect_descriptions.push("Sets enemies ablaze".to_string());
                        } else {
                            effect_descriptions.push("May ignite targets".to_string());
                        }
                    }
                }
            }

            if !effect_descriptions.is_empty() {
                description_parts.push(effect_descriptions.join(", ") + ".");
            }
        }

        self.description = description_parts.join(" ");
    }
}

/// Helper function to generate a random weapon with tier-based rarity
pub fn generate_random_weapon(
    base_weapon: Weapon,
    base_name: &str,
    base_description: &str,
    zone_tier: u32, // 0-10, higher = better loot
    rand: &mut Rand,
) -> GeneratedWeapon {
    // Convert zone tier to tier bonus (0.0 to 0.3)
    let tier_bonus = (zone_tier as f32 / 10.0) * 0.3;

    GeneratedWeapon::generate_random(base_weapon, base_name, base_description, tier_bonus, rand)
}
