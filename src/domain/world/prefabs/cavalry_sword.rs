use super::Prefab;
use crate::{
    common::Palette,
    domain::{
        ApplyVisibilityEffects, Equippable, Item, Label, MeleeWeapon, NeedsStableId, SaveFlag,
    },
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub fn spawn_cavalry_sword(entity: Entity, world: &mut World, config: Prefab) {
    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(20, Palette::Yellow, Palette::Gray).layer(Layer::Objects),
        Label::new("Cavalry Sword"),
        Item::new(3.0),
        Equippable::weapon_one_handed(),
        MeleeWeapon::new(5, vec![crate::domain::MaterialType::Flesh]),
        ApplyVisibilityEffects,
        RecordZonePosition,
        SaveFlag,
        NeedsStableId,
        CleanupStatePlay,
    ));
}
