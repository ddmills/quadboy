pub struct GameFormulas;

impl GameFormulas {
    // XP System Constants (from user's provided formula)
    pub const XP_REQ_CAP: u32 = 4000;
    pub const XP_LVL_INTENSITY: u32 = 10;
    pub const XP_BASE_GAIN: u32 = 120;
    pub const XP_SPREAD: u32 = 8;
    pub const XP_POWER: f64 = 3.0;
    pub const MAX_LEVEL: u32 = 50;

    /// Calculate XP gained when an attacker defeats a victim
    /// Formula: (Math.pow(Math.max(0, ((XP_SPREAD + 1) + (enemy_lvl - lvl))) / (XP_SPREAD + 1), XP_POWER) * XP_BASE_GAIN).floor()
    pub fn calculate_xp_gain(attacker_level: u32, victim_level: u32) -> u32 {
        let level_diff = victim_level as i32 - attacker_level as i32;
        let numerator = Self::XP_SPREAD as i32 + 1 + level_diff;
        let clamped_numerator = numerator.max(0) as f64;
        let denominator = (Self::XP_SPREAD + 1) as f64;

        let power_result = (clamped_numerator / denominator).powf(Self::XP_POWER);
        let xp_gain = (power_result * Self::XP_BASE_GAIN as f64).floor() as u32;

        xp_gain
    }

    /// Calculate XP required to reach the next level from current level
    pub fn xp_required_for_next_level(current_level: u32) -> u32 {
        if current_level >= Self::MAX_LEVEL {
            return u32::MAX;
        }

        let target_level = current_level + 1;
        let base_xp = 100u32;
        let level_multiplier = (target_level as f64 - 1.0).powf(1.5);
        let required_xp = (base_xp as f64 * level_multiplier).min(Self::XP_REQ_CAP as f64);

        required_xp as u32
    }
}
