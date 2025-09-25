use super::{Prefab, PrefabBuilder, SpawnValue};
use crate::{
    common::Palette,
    domain::{
        ApplyVisibilityEffects, AttributePoints, Attributes, Collider, ColliderFlags,
        DefaultMeleeAttack, DynamicEntity, Energy, EquipmentSlots, FactionId, FactionMember,
        Health, HitBlink, Inventory, Level, ModifierSource, MovementCapabilities, Player,
        StatModifier, StatModifiers, StatType, Stats, Vision,
    },
    rendering::{GlyphTextureId, Layer},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_player(entity: Entity, world: &mut World, config: Prefab) -> PrefabBuilder {
    // Get level from metadata, default to 2
    let level = config
        .metadata
        .get("level")
        .and_then(|v| match v {
            SpawnValue::Int(level) => Some(*level as u32),
            _ => None,
        })
        .unwrap_or(1);

    let hp_mod = StatModifier {
        value: 100,
        source: ModifierSource::Intrinsic {
            name: "GodMode".to_owned(),
        },
    };

    let mut mods = StatModifiers::new();
    mods.add_modifier(StatType::Fortitude, hp_mod);

    PrefabBuilder::new()
        .with_base_components(config.pos)
        .with_dynamic_tracking() // Player can move
        .with_glyph_and_texture(
            2,
            Palette::White,
            Palette::Blue,
            Layer::Actors,
            GlyphTextureId::Creatures,
        )
        .with_label("{Y|Cowboy}")
        .with_energy(-10)
        .with_health()
        .with_level(level)
        .with_attributes(Attributes::new(0, 0, 0, 0))
        .with_stats(Stats::new())
        .with_stat_modifiers(mods)
        .with_default_melee_attack(DefaultMeleeAttack::fire_fists())
        .with_inventory(50.0) // 50.0 capacity
        .with_component(Player)
        .with_component(EquipmentSlots::humanoid())
        .with_component(Vision::new(60))
        .with_component(ApplyVisibilityEffects)
        .with_component(Collider::new(
            ColliderFlags::SOLID | ColliderFlags::IS_ACTOR,
        ))
        .with_component(MovementCapabilities::terrestrial())
        .with_component(DynamicEntity)
        .with_component(CleanupStatePlay)
        .with_component(FactionMember::new(FactionId::Player))
        .with_component(AttributePoints::new(1)) // Level 1 = 5 + 1 = 6 points
        .with_component(Health::new_full()) // Will be set to proper max HP by health system
        .with_component(HitBlink::blinking(Palette::Green.into(), 0.5))
}
