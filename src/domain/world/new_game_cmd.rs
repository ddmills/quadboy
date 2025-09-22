use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::{
    cfg::SURFACE_LEVEL_Z,
    common::Palette,
    domain::{
        ApplyVisibilityEffects, AttributePoints, Attributes, Collider, DefaultMeleeAttack,
        DynamicEntity, Energy, EquipmentSlots, FactionId, FactionMember, GameSaveData, Health,
        Inventory, Label, Level, LoadZoneCommand, MovementCapabilities, Overworld, Player,
        PlayerPosition, PlayerSaveData, Prefab, PrefabId, Prefabs, StatModifiers, Stats,
        TerrainNoise, Vision, Zones,
    },
    engine::{Clock, StableId, StableIdRegistry, delete_save, save_game, serialize},
    rendering::{GameCamera, Glyph, GlyphTextureId, Layer, Position},
    states::{CleanupStatePlay, CurrentGameState, GameState},
};

pub struct NewGameCommand {
    pub save_name: String,
    pub seed: u32,
}

#[derive(Event)]
pub struct NewGameResult {
    pub success: bool,
}

impl Command<()> for NewGameCommand {
    fn apply(self, world: &mut World) {
        let result = self.execute_new_game(world);

        if let Some(mut events) = world.get_resource_mut::<Events<NewGameResult>>() {
            events.send(result);
        }
    }
}

impl NewGameCommand {
    fn execute_new_game(&self, world: &mut World) -> NewGameResult {
        delete_save(&self.save_name);

        let starting_position = Position::new(130, 233, SURFACE_LEVEL_Z);
        let start_zone = starting_position.zone_idx();

        let mut id_registry = StableIdRegistry::new();
        let player_id = id_registry.generate_id();

        let player_entity = world
            .spawn((
                starting_position.clone(),
                Glyph::new(2, Palette::White, Palette::Blue)
                    .layer(Layer::Actors)
                    .texture(GlyphTextureId::Creatures),
                Player,
                StableId::new(player_id),
                Inventory::new(50.0),
                EquipmentSlots::humanoid(),
                Vision::new(60),
                ApplyVisibilityEffects,
                Collider::new(
                    crate::domain::ColliderFlags::SOLID | crate::domain::ColliderFlags::IS_ACTOR,
                ),
                MovementCapabilities::terrestrial(),
                Energy::new(-10),
            ))
            .insert(Label::new("{Y|Cowboy}"))
            .insert(DefaultMeleeAttack::fists())
            .insert(Level::new(2))
            .insert(DynamicEntity) // Player can move
            .insert(CleanupStatePlay)
            .insert(FactionMember::new(FactionId::Player))
            .insert(Attributes::new(0, 0, 0, 0))
            .insert(AttributePoints::new(1)) // Level 1 = 5 + 1 = 6 points
            .insert(Stats::new())
            .insert(StatModifiers::new())
            .insert(Health::new_full()) // Will be set to proper max HP by health system
            .id();

        id_registry.register(player_entity, player_id);

        let mut camera = world.get_resource_mut::<GameCamera>().unwrap();
        camera.focus_on(starting_position.x, starting_position.y);

        world.insert_resource(PlayerPosition::from_position(&starting_position));
        world.insert_resource(Overworld::new(self.seed));
        world.insert_resource(TerrainNoise::new(self.seed));
        world.insert_resource(Clock::new(40000)); // 6:40am
        world.insert_resource(id_registry);
        world.insert_resource(Zones {
            player: start_zone,
            active: vec![start_zone],
            cache: HashMap::new(),
        });

        let _ = LoadZoneCommand(start_zone).apply(world);

        // Spawn starter items and add them to player's inventory (after StableIdRegistry is available)
        let starter_items = vec![
            PrefabId::NavyRevolver,
            PrefabId::LeverActionRifle,
            PrefabId::DoubleBarrelShotgun,
            PrefabId::Dynamite,
            PrefabId::Pickaxe,
            PrefabId::Hatchet,
        ];

        for item_id in starter_items {
            let config = Prefab::new(item_id, (0, 0, 0)); // Position doesn't matter for inventory items
            Prefabs::spawn_in_container(world, config, player_entity);
        }

        let serialized_player = serialize(player_entity, world);

        // Collect and serialize player's inventory items (same logic as save_game_cmd.rs)
        let mut inventory_items = vec![];
        let mut q_inventory = world.query::<&Inventory>();
        let id_registry = world.get_resource::<StableIdRegistry>().unwrap();

        if let Ok(inventory) = q_inventory.get(world, player_entity) {
            for item_id in inventory.item_ids.iter() {
                if let Some(item_entity) = id_registry.get_entity(*item_id) {
                    let serialized_item = serialize(item_entity, world);
                    inventory_items.push(serialized_item);
                }
            }
        }

        let player_save_data = PlayerSaveData {
            position: starting_position,
            entity: serialized_player,
            inventory_items,
        };

        let game_save_data = GameSaveData::new(player_save_data, 0.0, 0, self.seed);
        save_game(&game_save_data, &self.save_name);

        if let Some(mut game_state) = world.get_resource_mut::<CurrentGameState>() {
            game_state.next = GameState::Explore;
        }

        NewGameResult { success: true }
    }
}
