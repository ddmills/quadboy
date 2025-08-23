# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

### Native Build
```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo run                      # Build and run debug
cargo run --release            # Build and run release
```

### Web Build (WASM)
```bash
./build-web.sh                 # Debug web build
./build-web.sh --release       # Release web build
./build-web.sh -rs             # Release build with local server
./build-web.sh -rso            # Release build, serve, and open browser
```

### Code Quality
```bash
cargo fmt                      # Format code
cargo clippy                   # Run linter (configured via clippy.toml)
cargo check                    # Quick compile check
```

## Architecture Overview

QuadBoy is a Rust game built with **Macroquad** for graphics and **Bevy ECS** for entity management. The architecture follows a modular plugin-based design with clear separation of concerns.

### Core Systems
- **Engine**: Core application framework with plugin system, input handling, time management, and save/load functionality
- **Rendering**: Custom text-based rendering system with CRT shader effects, glyph batching, and layered rendering
- **Domain**: Game logic including player systems and world/zone management
- **States**: Game state management (main menu, play, explore, pause states). Also referred to as Screens.
- **UI**: Layout and user interface components

### Key Dependencies
- **macroquad**: Graphics framework and windowing
- **bevy_ecs**: Entity Component System
- **serde**: Serialization for save/load system
- Custom derive macro: `bevy_serializable_derive` for component serialization

### Entity Component System
The game uses Bevy ECS with a custom serialization system. Components must be registered with `SerializableComponentRegistry` for save/load functionality.

- Entities can be serialized/deserialized using `serialize`, `deserialize`, and `deserialize_all`functions
- Entities with the `SaveFlag` component will be saved when a zone is unloaded

### Rendering System
- Text-based rendering using custom glyph system
- Text glyphs have half-height (0.5f) of game glyphs (1.0f)
- Glyphs are batch rendered in layers
- Some Layers are in the world space and some are UI space
- CRT shader for retro aesthetic
- Asset loading for tilesets and textures located in `src/assets/`
- Glyphs use four colors: Foreground1 (fg1), Fourground2 (fg2), Outline (outline), and Background (bg)
- outline and bg should be rarely used
- fg1 should be the primary color
- fg2 should be the secondary color
- colors are defined in `palette.rs`. Use the PaletteEnum when possible.

### Zone/World System
The game implements a zone-based world system where different areas can be loaded/unloaded dynamically. Commands like `LoadZoneCommand` and `UnloadZoneCommand` manage this system.

- All zones have an index, which is a unique Id for the zone
- `projection.rs` contains helper methods for converting between different coordinate systems, and getting zone indices
- Use the `Zone::get_at` function to get a Vec of entities at a given position
- Use the `Zone::get_neighbors` function to get a Vec<Vec> of entities that neighbor a given position

### Clock System
- The game is turn-based
- Actors take actions that cost "Energy"
- When the Clock `tick` is incremented, actors gain energy.
- If all actors have negative energy, the world 'tick' is incremented until an actor has Zero energy
- Turn order is determined by the entity with the most energy.
- Energy can be a negative value

### Build Notes
- Web builds require `wasm32-unknown-unknown` target and `wasm-bindgen`
- The build script handles WASM compilation and Macroquad-specific patches
- Clippy configuration allows higher complexity thresholds for game code

## Component Serialization System
When creating new components that need to be saved/loaded:
1. Add `#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]` to the component
2. Register the component in `main.rs` with `reg.register::<YourComponent>()`
3. Components without the SerializableComponent trait won't be saved with entities

Important serializable components:
- Player, Position, Energy, Glyph, Label, Collider
- RecordZonePosition (marks entities that track their zone)
- CleanupStatePlay/CleanupStateExplore (cleanup markers)

## Turn-Based Energy System
- Energy determines turn order - entity with highest energy acts next
- When all entities have negative energy, time advances
- Player input only accepted during player's turn (`TurnState::is_players_turn`)
- AI turns are processed immediately in a loop (max 100 iterations)
- Energy costs: Move=100, Wait=50, Sleep=1000, Attack=150

## Zone Management
- World is divided into zones of size 80x40 (ZONE_SIZE)
- Map size is 12x8x20 zones (MAP_SIZE)
- Zones can be Active, Dormant, or Unloaded
- Zone::entities is a HashGrid for spatial indexing of entities
- Use `Zone::get_at()` to get entities at world coordinates
- Use `Zone::get_neighbors()` to get entities in cardinal directions
- Always check if a zone is loaded before allowing entity movement into it

## Prefab System
The game uses a command-based prefab system for entity spawning:
- `PrefabId` enum defines available prefab types (PineTree, Boulder, Bandit, StairUp, StairDown)
- `SpawnConfig` configures spawn parameters: position as (usize, usize, usize), zone entity, variants, custom labels/colors
- `SpawnPrefabCommand` provides deferred entity spawning through Commands
- `Prefabs` resource manages function registry for spawn functions
- Two spawn methods: `spawn()` for Commands-based (deferred), `spawn_world()` for World-based (immediate)
- Spawn functions signature: `fn(Entity, &mut World, SpawnConfig)` - modify entity in-place
- Located in `src/domain/world/prefabs/` with separate files per prefab type
- Usage: `let config = SpawnConfig::new((x, y, z), zone_entity); Prefabs::spawn_world(world, PrefabId::PineTree, config)`

## Save System Architecture
- GameSaveData contains: player data, timestamp, and clock tick
- PlayerSaveData contains: position and serialized entity
- Zones are saved separately as JSON files
- Save files stored in `saves/{save_name}/` directory
- Player entity is fully serialized/deserialized with all components
- Clock tick is preserved to maintain game time consistency

## Common Patterns and Helpers

### Coordinate System Helpers
- `world_to_zone_idx(x, y, z)` - Convert world coords to zone index
- `world_to_zone_local(x, y)` - Convert world coords to local zone coords
- `zone_xyz(idx)` - Convert zone index to zone coordinates
- `Position::world()` - Get world coordinates as (usize, usize, usize)

### Query Patterns
- Use `ParamSet` when you need both mutable and immutable access to the same component
- Use `Zone::get_at()` instead of iterating all entities for collision checks
- Prefer spatial queries using Zone's entity grid over global queries

## Testing and Debugging

### Debug Features
- Press 'G' to spawn a wall at player position (for testing)
- PlayerDebug component shows position info
- FPS display can be toggled
- CRT shader effects can be configured

### Common Issues
- "NoEntities" panic usually means Player component isn't registered or saved
- Borrow checker issues with World often need ParamSet or resource scoping
- Zone loading issues can cause entities to disappear - always check zone status

## Performance Considerations
- AI processes all turns immediately (not frame-by-frame)
- Use Zone's spatial indexing for collision detection
- Limit game loop to 100 iterations to prevent infinite loops
- Entity serialization includes only registered components
- Zone unloading can optionally despawn entities (despawn_entities flag)

## Code Organization

### Module Structure
- **domain/systems**: Game logic systems (energy, AI, etc.)
- **domain/world**: World/zone generation and management
- **domain/components**: Component definitions
- **rendering**: All rendering-related systems and components
- **states**: Game state management (menu, play, pause, etc.)
- **engine**: Core framework, serialization, input handling

### Adding New Systems
1. Create system function in appropriate module
2. Register with SystemId if needed for run_system
3. Add to appropriate state plugin's update chain
4. Consider energy consumption for player actions

## Code style
- Query system parameters should be prefixed with `q_`, for example, `q_position: Query<&Position>`. 
- Event Reader/Reader system parameters should be prefixed with `e_`, for example: `mut e_refresh_bitmask: EventWriter<RefreshBitmask>`
- bevy `Commands` parameters should be named `cmds`
- Leave very few, if any, comments
- prefer importing bevy prelude over individual parts. eg `use bevy_ecs::prelude::*;`

### Formatting Text Glyphs in game
- Text can be stylized in the game, for example: `{R-G-B-y repeat|Hello World}` will output the text "Hello World" in Red, Green, Blue, and Dark Yellow colors, repeating.
- The colors are defined in `palette.rs`. `get_seq_color` is the mapping between a character and the color.
- `PaletteSequenceType` are different ways the colors can be applied to the text.
