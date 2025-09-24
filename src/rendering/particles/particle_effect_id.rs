use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParticleEffectId {
    // Gun-based projectile effects
    Pistol { bullet_speed: f32 },
    Rifle { bullet_speed: f32 },
    Shotgun { bullet_speed: f32 },

    // Melee effects (for future expansion)
    BladeSlash,
    BluntImpact,

    // Material hit effects - directional impacts with material-specific properties
    HitStone,
    HitWood,
    HitFlesh,

    // Special weapon effects
    Explosion { radius: f32 },

    // Magical/special effects (for future expansion)
    FireBolt,
    IceShard,
    LightningBolt,
}

impl ParticleEffectId {
    // Default configurations for common weapon types
    pub fn default_pistol() -> Self {
        Self::Pistol { bullet_speed: 60.0 }
    }

    pub fn default_rifle() -> Self {
        Self::Rifle { bullet_speed: 80.0 }
    }

    pub fn default_shotgun() -> Self {
        Self::Shotgun { bullet_speed: 45.0 }
    }

    pub fn default_explosion(radius: f32) -> Self {
        Self::Explosion { radius }
    }

    /// Get the bullet speed for projectile-based effects
    pub fn get_bullet_speed(&self) -> f32 {
        match self {
            ParticleEffectId::Pistol { bullet_speed } => *bullet_speed,
            ParticleEffectId::Rifle { bullet_speed } => *bullet_speed,
            ParticleEffectId::Shotgun { bullet_speed } => *bullet_speed,
            _ => 60.0, // Default speed for non-projectile effects
        }
    }
}
