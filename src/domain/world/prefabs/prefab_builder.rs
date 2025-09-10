use super::Prefab;
use crate::{
    common::Palette,
    domain::{
        ApplyVisibilityEffects, BitmaskGlyph, BitmaskStyle, Collider, Destructible, Energy,
        Equippable, Health, HideWhenNotVisible, Inventory, InventoryAccessible,
        Item, Label, LightBlocker, LightSource, Lightable, LootDrop, MaterialType, MeleeWeapon,
        NeedsStableId, RangedWeapon, SaveFlag, StackCount, Stackable, StackableType, StairDown,
        StairUp, VisionBlocker,
    },
    rendering::{AnimatedGlyph, Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

pub struct PrefabBuilder<'a> {
    entity: Entity,
    world: &'a mut World,
    position: Position,
}

impl<'a> PrefabBuilder<'a> {
    pub fn new(entity: Entity, world: &'a mut World, config: &Prefab) -> Self {
        Self {
            entity,
            world,
            position: Position::new_world(config.pos),
        }
    }

    pub fn with_base_components(self) -> Self {
        self.world.entity_mut(self.entity).insert((
            self.position.clone(),
            RecordZonePosition,
            ApplyVisibilityEffects,
            SaveFlag,
            CleanupStatePlay,
        ));
        self
    }

    pub fn with_glyph(self, glyph_char: usize, fg1: Palette, fg2: Palette, layer: Layer) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(Glyph::new(glyph_char, fg1, fg2).layer(layer));
        self
    }

    pub fn with_animated_glyph(
        self,
        frames: Vec<usize>,
        speed_hz: f32,
        fg1: Palette,
        fg2: Palette,
        layer: Layer,
        loop_animation: bool,
    ) -> Self {
        let base_glyph = Glyph::new(frames[0], fg1, fg2).layer(layer);
        let animated_glyph = AnimatedGlyph::new(frames, speed_hz).with_loop(loop_animation);

        self.world
            .entity_mut(self.entity)
            .insert((base_glyph, animated_glyph));
        self
    }

    pub fn with_label(self, label: &str) -> Self {
        self.world.entity_mut(self.entity).insert(Label::new(label));
        self
    }

    pub fn with_collider(self) -> Self {
        self.world.entity_mut(self.entity).insert(Collider);
        self
    }

    pub fn with_vision_blocker(self) -> Self {
        self.world.entity_mut(self.entity).insert(VisionBlocker);
        self
    }

    pub fn with_light_blocker(self) -> Self {
        self.world.entity_mut(self.entity).insert(LightBlocker);
        self
    }

    pub fn with_destructible(self, health: i32, material: MaterialType) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(Destructible::new(health, material));
        self
    }

    pub fn with_bitmask(self, style: BitmaskStyle) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(BitmaskGlyph::new(style));
        self
    }

    pub fn with_energy(self, energy: i32) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(Energy::new(energy));
        self
    }

    pub fn with_health(self, health: i32) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(Health::new(health));
        self
    }

    pub fn with_hide_when_not_visible(self) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(HideWhenNotVisible);
        self
    }

    pub fn with_item(self, weight: f32) -> Self {
        self.world.entity_mut(self.entity).insert(Item::new(weight));
        self
    }

    pub fn with_equippable(self, equippable: Equippable) -> Self {
        self.world.entity_mut(self.entity).insert(equippable);
        self
    }

    pub fn with_melee_weapon(self, weapon: MeleeWeapon) -> Self {
        self.world.entity_mut(self.entity).insert(weapon);
        self
    }

    pub fn with_ranged_weapon(self, weapon: RangedWeapon) -> Self {
        self.world.entity_mut(self.entity).insert(weapon);
        self
    }

    pub fn with_needs_stable_id(self) -> Self {
        self.world.entity_mut(self.entity).insert(NeedsStableId);
        self
    }

    pub fn with_inventory(self, capacity: f32) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(Inventory::new(capacity));
        self
    }

    pub fn with_inventory_accessible(self) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(InventoryAccessible);
        self
    }

    pub fn with_stair_up(self) -> Self {
        self.world.entity_mut(self.entity).insert(StairUp);
        self
    }

    pub fn with_stair_down(self) -> Self {
        self.world.entity_mut(self.entity).insert(StairDown);
        self
    }

    pub fn with_loot_drop(self, loot_drop: LootDrop) -> Self {
        self.world.entity_mut(self.entity).insert(loot_drop);
        self
    }

    pub fn with_stackable(self, stack_type: StackableType, count: u32) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert((Stackable::new(stack_type), StackCount::new(count)));
        self
    }

    pub fn with_light_source(self, light: LightSource) -> Self {
        self.world.entity_mut(self.entity).insert(light);
        self
    }


    pub fn with_lightable(self) -> Self {
        self.world.entity_mut(self.entity).insert(Lightable::new());
        self
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}
