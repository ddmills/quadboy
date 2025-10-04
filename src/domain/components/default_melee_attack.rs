use crate::{
    domain::components::{
        destructible::MaterialType, hit_effect::HitEffect, weapon::Weapon,
        weapon_family::WeaponFamily, weapon_type::WeaponType,
    },
    engine::{AudioKey, SerializableComponent},
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct DefaultMeleeAttack {
    pub weapon: Weapon,
}

impl DefaultMeleeAttack {
    pub fn new(
        damage: i32,
        can_damage: Vec<MaterialType>,
        attack_name: &str,
        weapon_family: WeaponFamily,
        attack_verb: &str,
        attack_noun: &str,
    ) -> Self {
        let weapon = Weapon::new_melee(
            damage.to_string(),
            can_damage,
            weapon_family,
            attack_verb.to_string(),
            attack_noun.to_string(),
        );

        Self { weapon }
    }

    pub fn with_hit_effects(
        damage: i32,
        can_damage: Vec<MaterialType>,
        attack_name: &str,
        weapon_family: WeaponFamily,
        hit_effects: Vec<HitEffect>,
        attack_verb: &str,
        attack_noun: &str,
    ) -> Self {
        let mut weapon = Weapon::new_melee(
            damage.to_string(),
            can_damage,
            weapon_family,
            attack_verb.to_string(),
            attack_noun.to_string(),
        );
        weapon.hit_effects = hit_effects;

        Self { weapon }
    }

    pub fn fists() -> Self {
        let mut attack = Self::new(
            2,
            vec![MaterialType::Flesh],
            "Fists",
            WeaponFamily::Unarmed,
            "punches",
            "punch",
        );
        attack.weapon.melee_audio = Some(AudioKey::Punch1);
        attack
    }

    pub fn fists_with_knockback() -> Self {
        let mut attack = Self::with_hit_effects(
            2,
            vec![MaterialType::Flesh],
            "Fists",
            WeaponFamily::Unarmed,
            vec![HitEffect::Knockback {
                strength: 0.5,
                chance: 1.0,
            }], // Always knockback at half strength
            "punches",
            "punch",
        );
        attack.weapon.melee_audio = Some(AudioKey::Punch1);
        attack
    }

    pub fn claw_swipe() -> Self {
        Self::new(
            4,
            vec![MaterialType::Flesh, MaterialType::Wood],
            "Claw Swipe",
            WeaponFamily::Unarmed,
            "claws",
            "claw",
        )
    }

    pub fn venomous_bite() -> Self {
        Self::with_hit_effects(
            3,
            vec![MaterialType::Flesh],
            "Venomous Bite",
            WeaponFamily::Unarmed,
            vec![HitEffect::Poison {
                damage_per_tick: 2,
                duration_ticks: 1000,
                chance: 1.0, // Always apply poison
            }],
            "bites",
            "bite",
        )
    }
    pub fn fire_fists() -> Self {
        let mut attack = Self::with_hit_effects(
            3,
            vec![MaterialType::Flesh],
            "Fire Fists",
            WeaponFamily::Unarmed,
            vec![HitEffect::Bleeding {
                damage_per_tick: 2,
                duration_ticks: 1000,
                chance: 1.0, // Always apply poison
                can_stack: false,
            }],
            "strikes",
            "strike",
        );
        attack.weapon.melee_audio = Some(AudioKey::Punch1);
        attack
    }

    pub fn bite() -> Self {
        Self::new(
            3,
            vec![MaterialType::Flesh],
            "Bite",
            WeaponFamily::Unarmed,
            "bites",
            "bite",
        )
    }

    pub fn wing_buffet() -> Self {
        Self::new(
            1,
            vec![MaterialType::Flesh],
            "Wing Buffet",
            WeaponFamily::Unarmed,
            "buffets",
            "buffet",
        )
    }

    pub fn electric_touch() -> Self {
        Self::new(
            2,
            vec![MaterialType::Flesh],
            "Electric Touch",
            WeaponFamily::Unarmed,
            "electrifies",
            "shock",
        )
    }

    pub fn nibble() -> Self {
        Self::with_hit_effects(
            1,
            vec![MaterialType::Flesh],
            "Nibble",
            WeaponFamily::Unarmed,
            vec![HitEffect::Bleeding {
                damage_per_tick: 1,
                duration_ticks: 500,
                chance: 1.0,
                can_stack: true,
            }],
            "nibbles",
            "nibble",
        )
    }

    pub fn mandible_crush() -> Self {
        Self::with_hit_effects(
            5,
            vec![MaterialType::Flesh, MaterialType::Wood],
            "Burning Mandible Crush",
            WeaponFamily::Unarmed,
            vec![HitEffect::Burning {
                damage_per_tick: 1,
                duration_ticks: 500,
                chance: 1.0,
            }],
            "crushes",
            "crush",
        )
    }
}
