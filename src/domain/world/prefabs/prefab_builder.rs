use super::Prefab;
use crate::{
    common::Palette,
    domain::{
        ApplyVisibilityEffects, AttributePoints, Attributes, BitmaskGlyph, BitmaskStyle, Collider,
        Consumable, ConsumableEffect, CreatureType, DefaultMeleeAttack, DefaultRangedAttack,
        Description, Destructible, DynamicEntity, Energy, EquipmentSlots, Equippable,
        ExplosiveProperties, FactionMember, Health, HideWhenNotVisible, Inventory,
        InventoryAccessible, Item, Label, Level, LightBlocker, LightSource, Lightable, LootDrop,
        MaterialType, MovementCapabilities, NeedsStableId, Player, SaveFlag, StackCount, Stackable,
        StackableType, StairDown, StairUp, StatModifiers, StaticEntity, StaticEntitySpawnedEvent,
        Stats, Throwable, Vision, VisionBlocker, Weapon, components::ai_controller::AiController,
    },
    engine::AudioKey,
    rendering::{AnimatedGlyph, Glyph, GlyphTextureId, Layer, Position},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, world::World};

#[derive(Clone)]
pub enum PrefabComponent {
    Position(Position),
    StaticEntity(StaticEntity),
    DynamicEntity(DynamicEntity),
    ApplyVisibilityEffects(ApplyVisibilityEffects),
    SaveFlag(SaveFlag),
    CleanupStatePlay(CleanupStatePlay),
    Glyph(Glyph),
    AnimatedGlyph(AnimatedGlyph),
    Label(Label),
    Description(Description),
    Item(Item),
    Collider(Collider),
    VisionBlocker(VisionBlocker),
    LightBlocker(LightBlocker),
    Destructible(Destructible),
    BitmaskGlyph(BitmaskGlyph),
    Energy(Energy),
    Health(Health),
    HideWhenNotVisible(HideWhenNotVisible),
    Inventory(Inventory),
    InventoryAccessible(InventoryAccessible),
    NeedsStableId(NeedsStableId),
    Equippable(Equippable),
    Weapon(Weapon),
    DefaultMeleeAttack(DefaultMeleeAttack),
    DefaultRangedAttack(DefaultRangedAttack),
    Consumable(Consumable),
    Stackable(Stackable),
    StackCount(StackCount),
    Throwable(Throwable),
    LootDrop(LootDrop),
    StairUp(StairUp),
    StairDown(StairDown),
    CreatureType(CreatureType),
    Level(Level),
    Attributes(Attributes),
    Stats(Stats),
    StatModifiers(StatModifiers),
    LightSource(LightSource),
    Lightable(Lightable),
    Player(Player),
    EquipmentSlots(EquipmentSlots),
    Vision(Vision),
    MovementCapabilities(MovementCapabilities),
    FactionMember(FactionMember),
    AttributePoints(AttributePoints),
    ExplosiveProperties(ExplosiveProperties),
    AiController(AiController),
}

pub struct PrefabBuilder {
    components: Vec<PrefabComponent>,
    fire_event: bool,
}

impl PrefabBuilder {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            fire_event: true,
        }
    }

    pub fn with_base_components(mut self, pos: (usize, usize, usize)) -> Self {
        self.components
            .push(PrefabComponent::Position(Position::new_world(pos)));
        self.components
            .push(PrefabComponent::ApplyVisibilityEffects(
                ApplyVisibilityEffects,
            ));
        self.components.push(PrefabComponent::SaveFlag(SaveFlag));
        self.components
            .push(PrefabComponent::CleanupStatePlay(CleanupStatePlay));
        self
    }

    pub fn with_static_tracking(mut self) -> Self {
        self.components
            .push(PrefabComponent::StaticEntity(StaticEntity));
        self
    }

    pub fn with_dynamic_tracking(mut self) -> Self {
        self.components
            .push(PrefabComponent::DynamicEntity(DynamicEntity));
        self
    }

    pub fn with_glyph(
        mut self,
        glyph_char: usize,
        fg1: Palette,
        fg2: Palette,
        layer: Layer,
    ) -> Self {
        self.components.push(PrefabComponent::Glyph(
            Glyph::new(glyph_char, fg1, fg2).layer(layer),
        ));
        self
    }

    pub fn with_glyph_and_texture(
        mut self,
        glyph_char: usize,
        fg1: Palette,
        fg2: Palette,
        layer: Layer,
        texture_id: GlyphTextureId,
    ) -> Self {
        self.components.push(PrefabComponent::Glyph(
            Glyph::new(glyph_char, fg1, fg2)
                .layer(layer)
                .texture(texture_id),
        ));
        self
    }

    pub fn with_animated_glyph(
        mut self,
        frames: Vec<usize>,
        speed_hz: f32,
        fg1: Palette,
        fg2: Palette,
        layer: Layer,
        loop_animation: bool,
    ) -> Self {
        let base_glyph = Glyph::new(frames[0], fg1, fg2).layer(layer);
        let animated_glyph = AnimatedGlyph::new(frames, speed_hz).with_loop(loop_animation);
        self.components.push(PrefabComponent::Glyph(base_glyph));
        self.components
            .push(PrefabComponent::AnimatedGlyph(animated_glyph));
        self
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.components
            .push(PrefabComponent::Label(Label::new(label)));
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.components
            .push(PrefabComponent::Description(Description::new(description)));
        self
    }

    pub fn with_collider(mut self) -> Self {
        self.components
            .push(PrefabComponent::Collider(Collider::solid()));
        self
    }

    pub fn with_collider_flags(mut self, flags: crate::domain::ColliderFlags) -> Self {
        self.components
            .push(PrefabComponent::Collider(Collider::new(flags)));
        self
    }

    pub fn with_actor_collider(mut self) -> Self {
        self.components
            .push(PrefabComponent::Collider(Collider::new(
                crate::domain::ColliderFlags::IS_ACTOR,
            )));
        self
    }

    pub fn with_vision_blocker(mut self) -> Self {
        self.components
            .push(PrefabComponent::VisionBlocker(VisionBlocker));
        self
    }

    pub fn with_light_blocker(mut self) -> Self {
        self.components
            .push(PrefabComponent::LightBlocker(LightBlocker));
        self
    }

    pub fn with_destructible(mut self, health: i32, material: MaterialType) -> Self {
        self.components
            .push(PrefabComponent::Destructible(Destructible::new(
                health, material,
            )));
        self
    }

    pub fn with_bitmask(mut self, style: BitmaskStyle) -> Self {
        self.components
            .push(PrefabComponent::BitmaskGlyph(BitmaskGlyph::new(style)));
        self
    }

    pub fn with_energy(mut self, energy: i32) -> Self {
        self.components
            .push(PrefabComponent::Energy(Energy::new(energy)));
        self.components
            .push(PrefabComponent::DynamicEntity(DynamicEntity)); // Auto-add DynamicEntity for creatures
        self
    }

    pub fn with_health(mut self) -> Self {
        self.components
            .push(PrefabComponent::Health(Health::new_full()));
        self
    }

    pub fn with_hide_when_not_visible(mut self) -> Self {
        self.components
            .push(PrefabComponent::HideWhenNotVisible(HideWhenNotVisible));
        self
    }

    pub fn with_item(mut self, weight: f32) -> Self {
        self.components
            .push(PrefabComponent::Item(Item::new(weight)));
        self
    }

    pub fn with_equippable(mut self, equippable: Equippable) -> Self {
        self.components
            .push(PrefabComponent::Equippable(equippable));
        self
    }

    pub fn with_default_melee_attack(mut self, attack: DefaultMeleeAttack) -> Self {
        self.components
            .push(PrefabComponent::DefaultMeleeAttack(attack));
        self
    }

    pub fn with_weapon(mut self, weapon: Weapon) -> Self {
        self.components.push(PrefabComponent::Weapon(weapon));
        self
    }

    pub fn with_needs_stable_id(mut self) -> Self {
        self.components
            .push(PrefabComponent::NeedsStableId(NeedsStableId));
        self
    }

    pub fn with_inventory(mut self, capacity: f32) -> Self {
        self.components
            .push(PrefabComponent::Inventory(Inventory::new(capacity)));
        self
    }

    pub fn with_inventory_accessible(mut self) -> Self {
        self.components
            .push(PrefabComponent::InventoryAccessible(InventoryAccessible));
        self
    }

    pub fn with_stair_up(mut self) -> Self {
        self.components.push(PrefabComponent::StairUp(StairUp));
        self
    }

    pub fn with_stair_down(mut self) -> Self {
        self.components.push(PrefabComponent::StairDown(StairDown));
        self
    }

    pub fn with_loot_drop(mut self, loot_drop: LootDrop) -> Self {
        self.components.push(PrefabComponent::LootDrop(loot_drop));
        self
    }

    pub fn with_stackable(mut self, stack_type: StackableType, count: u32) -> Self {
        self.components
            .push(PrefabComponent::Stackable(Stackable::new(stack_type)));
        self.components
            .push(PrefabComponent::StackCount(StackCount::new(count)));
        self
    }

    pub fn with_light_source(mut self, light: LightSource) -> Self {
        self.components.push(PrefabComponent::LightSource(light));
        self
    }

    pub fn with_lightable_audio(
        mut self,
        light_audio: AudioKey,
        extinguish_audio: Option<AudioKey>,
    ) -> Self {
        let mut lightable = Lightable::new().with_light_audio(light_audio);
        if let Some(extinguish) = extinguish_audio {
            lightable = lightable.with_extinguish_audio(extinguish);
        }
        self.components.push(PrefabComponent::Lightable(lightable));
        self
    }

    pub fn with_level(mut self, level: u32) -> Self {
        self.components
            .push(PrefabComponent::Level(Level::new(level)));
        self
    }

    pub fn with_attributes(mut self, attributes: Attributes) -> Self {
        self.components
            .push(PrefabComponent::Attributes(attributes));
        self
    }

    pub fn with_stats(mut self, stats: Stats) -> Self {
        self.components.push(PrefabComponent::Stats(stats));
        self
    }

    pub fn with_stat_modifiers(mut self, stat_modifiers: StatModifiers) -> Self {
        self.components
            .push(PrefabComponent::StatModifiers(stat_modifiers));
        self
    }

    pub fn with_creature_type(mut self, creature_type: CreatureType) -> Self {
        self.components
            .push(PrefabComponent::CreatureType(creature_type));
        self
    }

    pub fn with_component<T: bevy_ecs::component::Component>(mut self, component: T) -> Self {
        // Handle specific component types that are in our enum
        use std::any::Any;
        let component_any = &component as &dyn Any;

        if let Some(player) = component_any.downcast_ref::<Player>() {
            self.components
                .push(PrefabComponent::Player(player.clone()));
        } else if let Some(equipment_slots) = component_any.downcast_ref::<EquipmentSlots>() {
            self.components
                .push(PrefabComponent::EquipmentSlots(equipment_slots.clone()));
        } else if let Some(vision) = component_any.downcast_ref::<Vision>() {
            self.components
                .push(PrefabComponent::Vision(vision.clone()));
        } else if let Some(movement_caps) = component_any.downcast_ref::<MovementCapabilities>() {
            self.components
                .push(PrefabComponent::MovementCapabilities(movement_caps.clone()));
        } else if let Some(faction_member) = component_any.downcast_ref::<FactionMember>() {
            self.components
                .push(PrefabComponent::FactionMember(faction_member.clone()));
        } else if let Some(attribute_points) = component_any.downcast_ref::<AttributePoints>() {
            self.components
                .push(PrefabComponent::AttributePoints(attribute_points.clone()));
        } else if let Some(collider) = component_any.downcast_ref::<Collider>() {
            self.components
                .push(PrefabComponent::Collider(collider.clone()));
        } else if let Some(health) = component_any.downcast_ref::<Health>() {
            self.components
                .push(PrefabComponent::Health(health.clone()));
        } else if let Some(dynamic_entity) = component_any.downcast_ref::<DynamicEntity>() {
            self.components
                .push(PrefabComponent::DynamicEntity(dynamic_entity.clone()));
        } else if let Some(cleanup) = component_any.downcast_ref::<CleanupStatePlay>() {
            self.components
                .push(PrefabComponent::CleanupStatePlay(cleanup.clone()));
        } else if let Some(apply_visibility) =
            component_any.downcast_ref::<ApplyVisibilityEffects>()
        {
            self.components
                .push(PrefabComponent::ApplyVisibilityEffects(
                    apply_visibility.clone(),
                ));
        } else if let Some(explosive_props) = component_any.downcast_ref::<ExplosiveProperties>() {
            self.components.push(PrefabComponent::ExplosiveProperties(
                explosive_props.clone(),
            ));
        } else if let Some(default_ranged_attack) =
            component_any.downcast_ref::<DefaultRangedAttack>()
        {
            self.components.push(PrefabComponent::DefaultRangedAttack(
                default_ranged_attack.clone(),
            ));
        } else if let Some(ai_controller) = component_any.downcast_ref::<AiController>() {
            self.components
                .push(PrefabComponent::AiController(ai_controller.clone()));
        }
        // If component type not handled, it's silently ignored for now
        self
    }

    pub fn with_movement_capabilities(mut self, _flags: crate::domain::MovementFlags) -> Self {
        // TODO: Add MovementFlags to PrefabComponent enum if needed
        self
    }

    pub fn with_consumable(mut self, effect: ConsumableEffect, consume_on_use: bool) -> Self {
        self.components
            .push(PrefabComponent::Consumable(Consumable::new(
                effect,
                consume_on_use,
            )));
        self
    }

    pub fn with_throwable(mut self, base_range: usize) -> Self {
        self.components
            .push(PrefabComponent::Throwable(Throwable::new(
                base_range, '?', 0xFFFFFF,
            )));
        self
    }

    pub fn with_throwable_char(
        mut self,
        base_range: usize,
        particle_char: char,
        throwable_fg1: u32,
    ) -> Self {
        self.components
            .push(PrefabComponent::Throwable(Throwable::new(
                base_range,
                particle_char,
                throwable_fg1,
            )));
        self
    }

    pub fn for_container(mut self) -> Self {
        // Remove Position and StaticEntity components for container items
        self.components.retain(|component| {
            !matches!(
                component,
                PrefabComponent::Position(_) | PrefabComponent::StaticEntity(_)
            )
        });
        self.fire_event = false;
        self
    }

    fn has_position(&self) -> bool {
        self.components
            .iter()
            .any(|c| matches!(c, PrefabComponent::Position(_)))
    }

    fn has_static_entity(&self) -> bool {
        self.components
            .iter()
            .any(|c| matches!(c, PrefabComponent::StaticEntity(_)))
    }

    fn get_position(&self) -> Option<Position> {
        self.components.iter().find_map(|c| {
            if let PrefabComponent::Position(pos) = c {
                Some(pos.clone())
            } else {
                None
            }
        })
    }

    fn get_collider_flags(&self) -> Option<crate::domain::ColliderFlags> {
        self.components.iter().find_map(|c| {
            if let PrefabComponent::Collider(collider) = c {
                Some(collider.flags)
            } else {
                None
            }
        })
    }

    pub fn build(self, entity: Entity, world: &mut World) -> Entity {
        let mut entity_mut = world.entity_mut(entity);

        // Apply all components
        for component in &self.components {
            match component {
                PrefabComponent::Position(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::StaticEntity(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::DynamicEntity(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::ApplyVisibilityEffects(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::SaveFlag(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::CleanupStatePlay(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Glyph(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::AnimatedGlyph(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Label(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Description(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Item(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Collider(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::VisionBlocker(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::LightBlocker(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Destructible(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::BitmaskGlyph(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Energy(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Health(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::HideWhenNotVisible(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Inventory(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::InventoryAccessible(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::NeedsStableId(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Equippable(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Weapon(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::DefaultMeleeAttack(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::DefaultRangedAttack(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Consumable(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Stackable(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::StackCount(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Throwable(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::LootDrop(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::StairUp(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::StairDown(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::CreatureType(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Level(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Attributes(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Stats(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::StatModifiers(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::LightSource(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Lightable(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Player(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::EquipmentSlots(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::Vision(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::MovementCapabilities(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::FactionMember(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::AttributePoints(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::ExplosiveProperties(c) => {
                    entity_mut.insert(c.clone());
                }
                PrefabComponent::AiController(c) => {
                    entity_mut.insert(c.clone());
                }
            }
        }

        drop(entity_mut);

        // Fire event if appropriate
        if self.fire_event && self.has_position() && self.has_static_entity() {
            if let Some(position) = self.get_position() {
                let collider_flags = self.get_collider_flags();
                world.send_event(StaticEntitySpawnedEvent {
                    entity,
                    position,
                    collider_flags,
                });
            }
        }

        entity
    }
}
