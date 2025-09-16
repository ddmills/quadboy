use super::Prefab;
use crate::{
    common::Palette,
    domain::{
        ApplyVisibilityEffects, Attributes, BitmaskGlyph, BitmaskStyle, Collider, Consumable,
        ConsumableEffect, CreatureType, DefaultMeleeAttack, Description, Destructible, Energy,
        Equippable, Health, HideWhenNotVisible, Inventory, InventoryAccessible, Item, Label, Level,
        LightBlocker, LightSource, Lightable, LootDrop, MaterialType, NeedsStableId, SaveFlag,
        StackCount, Stackable, StackableType, StairDown, StairUp, StatModifiers, Stats,
        VisionBlocker, Weapon,
    },
    rendering::{AnimatedGlyph, Glyph, GlyphTextureId, Layer, Position, RecordZonePosition},
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

    pub fn with_glyph_and_texture(
        self,
        glyph_char: usize,
        fg1: Palette,
        fg2: Palette,
        layer: Layer,
        texture_id: GlyphTextureId,
    ) -> Self {
        self.world.entity_mut(self.entity).insert(
            Glyph::new(glyph_char, fg1, fg2)
                .layer(layer)
                .texture(texture_id),
        );
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

    pub fn with_description(self, description: &str) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(Description::new(description));
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

    pub fn with_health(self) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(Health::new_full());
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

    pub fn with_default_melee_attack(self, attack: DefaultMeleeAttack) -> Self {
        self.world.entity_mut(self.entity).insert(attack);
        self
    }

    pub fn with_weapon(self, weapon: Weapon) -> Self {
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

    pub fn with_level(self, level: u32) -> Self {
        self.world.entity_mut(self.entity).insert(Level::new(level));
        self
    }

    pub fn with_attributes(self, attributes: Attributes) -> Self {
        self.world.entity_mut(self.entity).insert(attributes);
        self
    }

    pub fn with_stats(self, stats: Stats) -> Self {
        self.world.entity_mut(self.entity).insert(stats);
        self
    }

    pub fn with_stat_modifiers(self, stat_modifiers: StatModifiers) -> Self {
        self.world.entity_mut(self.entity).insert(stat_modifiers);
        self
    }

    pub fn with_creature_type(self, creature_type: CreatureType) -> Self {
        self.world.entity_mut(self.entity).insert(creature_type);
        self
    }

    pub fn with_consumable(self, effect: ConsumableEffect, consume_on_use: bool) -> Self {
        self.world
            .entity_mut(self.entity)
            .insert(Consumable::new(effect, consume_on_use));
        self
    }

    pub fn with_component<C: bevy_ecs::component::Component>(self, component: C) -> Self {
        self.world.entity_mut(self.entity).insert(component);
        self
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}
