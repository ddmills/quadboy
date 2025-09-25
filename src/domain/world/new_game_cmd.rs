use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::{
    cfg::SURFACE_LEVEL_Z,
    common::Palette,
    domain::{
        ApplyVisibilityEffects, AttributePoints, Attributes, Collider, DefaultMeleeAttack,
        DynamicEntity, Energy, EquipItemAction, EquipmentSlots, FactionId, FactionMember,
        GameSaveData, Health, Inventory, Label, Level, LoadZoneCommand, MovementCapabilities,
        NeedsStableId, Overworld, Player, PlayerPosition, PlayerSaveData, Prefab, PrefabId,
        Prefabs, StatModifiers, Stats, TerrainNoise, Vision, Zones,
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

        let starting_position = Position::new(196, 204, SURFACE_LEVEL_Z);
        let start_zone = starting_position.zone_idx();

        let id_registry = StableIdRegistry::new();
        world.insert_resource(id_registry);

        // Create player using prefab system
        let player_config = Prefab::new(
            PrefabId::Player,
            (
                starting_position.x as usize,
                starting_position.y as usize,
                starting_position.z as usize,
            ),
        );
        let player_entity = Prefabs::spawn_world(world, player_config);

        // Manually assign StableId to player so we can add items to inventory
        let player_stable_id = {
            let mut stable_id_registry = world.resource_mut::<StableIdRegistry>();
            let id = stable_id_registry.generate_id();
            stable_id_registry.register(player_entity, id);
            id
        };

        world
            .entity_mut(player_entity)
            .insert(player_stable_id)
            .remove::<NeedsStableId>();

        let mut camera = world.get_resource_mut::<GameCamera>().unwrap();
        camera.focus_on(starting_position.x, starting_position.y);

        world.insert_resource(PlayerPosition::from_position(&starting_position));
        world.insert_resource(Overworld::new(self.seed));
        world.insert_resource(TerrainNoise::new(self.seed));
        world.insert_resource(Clock::new(40000)); // 6:40am
        // world.insert_resource(Clock::new(100)); // 6:40am
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
            PrefabId::Overcoat,
            PrefabId::SteelToeBoots,
        ];

        let mut spawned_items = Vec::new();
        for item_id in starter_items {
            let config = Prefab::new(item_id.clone(), (0, 0, 0)); // Position doesn't matter for inventory items
            let item_entity = Prefabs::spawn_in_container(world, config, player_entity);
            spawned_items.push((item_id, item_entity));
        }

        // Auto-equip specific items
        let items_to_equip = vec![
            PrefabId::NavyRevolver,
            PrefabId::Overcoat,
            PrefabId::SteelToeBoots,
        ];

        for (item_id, item_entity) in spawned_items {
            if items_to_equip.contains(&item_id) {
                // Get the item's stable ID
                if let Some(item_stable_id) = world.get::<StableId>(item_entity) {
                    let equip_action = EquipItemAction {
                        entity_id: player_stable_id.0,
                        item_id: item_stable_id.0,
                    };
                    equip_action.apply(world);
                }
            }
        }

        let serialized_player = serialize(player_entity, world);

        // Collect and serialize player's inventory items (same logic as save_game_cmd.rs)
        let mut inventory_items = vec![];
        let mut q_inventory = world.query::<&Inventory>();
        let id_registry = world.get_resource::<StableIdRegistry>().unwrap();

        if let Ok(inventory) = q_inventory.get(world, player_entity) {
            for item_id in inventory.item_ids.iter() {
                if let Some(item_entity) = id_registry.get_entity(StableId(*item_id)) {
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
