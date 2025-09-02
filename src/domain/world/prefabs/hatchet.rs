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

pub fn spawn_hatchet(entity: Entity, world: &mut World, config: Prefab) {
    world.entity_mut(entity).insert((
        Position::new_world(config.pos),
        Glyph::new(21, Palette::Red, Palette::Gray).layer(Layer::Objects),
        Label::new("Hatchet"),
        Item::new(2.0),
        Equippable::tool(),
        MeleeWeapon::hatchet(),
        ApplyVisibilityEffects,
        RecordZonePosition,
        SaveFlag,
        NeedsStableId,
        CleanupStatePlay,
    ));
}
