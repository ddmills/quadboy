# QuadBoy Prefab System

## Overview

The QuadBoy Prefab System is a command-based entity spawning system that provides type safety, flexibility, and deferred execution. It uses simple enum variants without sub-types for easy maintenance and clear organization.

## Core Architecture

### Prefab Identification

```rust
// Simple prefab identifier - streamlined for clarity
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum PrefabId {
    PineTree,
    Boulder,
    Bandit,
    StairDown,
    StairUp,
}
```

### Spawn Configuration

```rust
// Configuration for spawn-time customization
#[derive(Clone, Debug)]
pub struct SpawnConfig {
    pub pos: (usize, usize, usize),     // Position as tuple
    pub zone_entity: Entity,
    pub variant: Option<String>,        // "large", "damaged", "magical", etc.
    pub level: Option<u32>,             // For enemies, items with levels
    pub custom_label: Option<String>,   // Override default label
    pub custom_color: Option<Palette>,  // Override default color
    pub metadata: HashMap<String, SpawnValue>, // Flexible key-value data
}

#[derive(Clone, Debug)]
pub enum SpawnValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}

impl SpawnConfig {
    pub fn new(pos: (usize, usize, usize), zone_entity: Entity) -> Self {
        Self {
            pos,
            zone_entity,
            variant: None,
            level: None,
            custom_label: None,
            custom_color: None,
            metadata: HashMap::new(),
        }
    }
    
    // Builder methods for easy configuration
    pub fn with_variant(mut self, variant: String) -> Self { /* ... */ }
    pub fn with_level(mut self, level: u32) -> Self { /* ... */ }
    pub fn with_custom_label(mut self, label: String) -> Self { /* ... */ }
    pub fn with_custom_color(mut self, color: Palette) -> Self { /* ... */ }
    pub fn with_metadata(mut self, key: String, value: SpawnValue) -> Self { /* ... */ }
}
```

### Command-Based Spawn System

```rust
// Command for deferred entity spawning
pub struct SpawnPrefabCommand {
    pub entity: Entity,
    pub prefab_id: PrefabId,
    pub config: SpawnConfig,
}

impl SpawnPrefabCommand {
    pub fn new(entity: Entity, prefab_id: PrefabId, config: SpawnConfig) -> Self {
        Self { entity, prefab_id, config }
    }
    
    pub fn execute(self, world: &mut World) -> Result<(), String> {
        let spawn_fn = {
            let prefabs = world
                .get_resource::<Prefabs>()
                .ok_or("Prefabs resource not found")?;

            *prefabs
                .spawn_functions
                .get(&self.prefab_id)
                .ok_or_else(|| format!("Unknown prefab type: {:?}", self.prefab_id))?
        };

        spawn_fn(self.entity, world, self.config);
        Ok(())
    }
}

// Prefabs resource with function registry
type SpawnFunction = fn(Entity, &mut World, SpawnConfig);

#[derive(Resource)]
pub struct Prefabs {
    pub spawn_functions: HashMap<PrefabId, SpawnFunction>,
}

impl Prefabs {
    pub fn new() -> Self {
        let mut system = Self {
            spawn_functions: HashMap::new(),
        };
        system.register_all_prefabs();
        system
    }
    
    fn register_all_prefabs(&mut self) {
        self.register(PrefabId::PineTree, spawn_pine_tree);
        self.register(PrefabId::Boulder, spawn_boulder);
        self.register(PrefabId::Bandit, spawn_bandit);
        self.register(PrefabId::StairDown, spawn_stair_down);
        self.register(PrefabId::StairUp, spawn_stair_up);
    }
    
    pub fn register(&mut self, id: PrefabId, spawn_fn: SpawnFunction) {
        self.spawn_functions.insert(id, spawn_fn);
    }
    
    // Commands-based spawning (deferred execution)
    pub fn spawn(&self, cmds: &mut Commands, prefab_id: PrefabId, config: SpawnConfig) -> Entity {
        let entity = cmds.spawn_empty().id();
        let command = SpawnPrefabCommand::new(entity, prefab_id, config);
        cmds.queue(move |world: &mut World| {
            if let Err(e) = command.execute(world) {
                eprintln!("Failed to spawn prefab: {}", e);
            }
        });
        entity
    }
    
    // World-based spawning (immediate execution)
    pub fn spawn_world(
        world: &mut World,
        prefab_id: PrefabId,
        config: SpawnConfig,
    ) -> Result<Entity, String> {
        let entity = world.spawn_empty().id();
        let command = SpawnPrefabCommand::new(entity, prefab_id, config);
        command.execute(world)?;
        Ok(entity)
    }
}
```

## Spawn Function Examples

### Simple Tree Spawning

```rust
// spawn_pine_tree.rs
use super::SpawnConfig;
use crate::{
    common::Palette,
    domain::{Label, SaveFlag, ZoneStatus},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, hierarchy::ChildOf, world::World};

pub fn spawn_pine_tree(entity: Entity, world: &mut World, config: SpawnConfig) {
    let label = config.custom_label.as_deref().unwrap_or("Pine Tree");
    let color = config.custom_color.unwrap_or(Palette::Green);
    let position = Position::new(config.pos.0, config.pos.1, config.pos.2);

    let mut entity_mut = world.entity_mut(entity);
    entity_mut.insert((
        position,
        Glyph::new(64, color, Palette::Clear).layer(Layer::Objects),
        Label::new(label),
        ChildOf(config.zone_entity),
        ZoneStatus::Dormant,
        RecordZonePosition,
        SaveFlag,
        CleanupStatePlay,
    ));
}
```

### Enemy Spawning with Energy System

```rust
// spawn_bandit.rs
use super::SpawnConfig;
use crate::{
    common::Palette,
    domain::{Energy, Label, SaveFlag, ZoneStatus},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, hierarchy::ChildOf, world::World};

pub fn spawn_bandit(entity: Entity, world: &mut World, config: SpawnConfig) {
    let label = config.custom_label.as_deref().unwrap_or("{R|Bandit}");
    let color = config.custom_color.unwrap_or(Palette::Red);
    let position = Position::new(config.pos.0, config.pos.1, config.pos.2);

    let mut entity_mut = world.entity_mut(entity);
    entity_mut.insert((
        position,
        Glyph::new(145, color, Palette::Clear).layer(Layer::Actors),
        Label::new(label),
        Energy::new(-100),
        ChildOf(config.zone_entity),
        ZoneStatus::Dormant,
        RecordZonePosition,
        SaveFlag,
        CleanupStatePlay,
    ));
}
```

### Stairs Spawning

```rust
// spawn_stair_up.rs
use super::SpawnConfig;
use crate::{
    common::Palette,
    domain::{Label, SaveFlag, StairUp, ZoneStatus},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
    states::CleanupStatePlay,
};
use bevy_ecs::{entity::Entity, hierarchy::ChildOf, world::World};

pub fn spawn_stair_up(entity: Entity, world: &mut World, config: SpawnConfig) {
    let label = config.custom_label.as_deref().unwrap_or("Stairs Up");
    let color = config.custom_color.unwrap_or(Palette::Brown);
    let position = Position::new(config.pos.0, config.pos.1, config.pos.2);

    let mut entity_mut = world.entity_mut(entity);
    entity_mut.insert((
        position,
        Glyph::new(60, color, Palette::Clear).layer(Layer::Objects),
        Label::new(label),
        StairUp,
        ChildOf(config.zone_entity),
        ZoneStatus::Dormant,
        RecordZonePosition,
        SaveFlag,
        CleanupStatePlay,
    ));
}
```

## Integration with Zone Generation

```rust
// Zone generation using the implemented prefab system
if rand.gen_bool(0.05) {
    let config = SpawnConfig::new((wpos.0, wpos.1, wpos.2), zone_entity_id);
    let _ = Prefabs::spawn_world(world, PrefabId::PineTree, config);
}

if rand.gen_bool(0.04) {
    let config = SpawnConfig::new((wpos.0, wpos.1, wpos.2), zone_entity_id);
    let _ = Prefabs::spawn_world(world, PrefabId::Bandit, config);
}

if rand.gen_bool(0.02) {
    let config = SpawnConfig::new((wpos.0, wpos.1, wpos.2), zone_entity_id);
    let _ = Prefabs::spawn_world(world, PrefabId::Boulder, config);
}

// Stairs are placed at specific locations
if room_coords.1 == min_y {
    let config = SpawnConfig::new((wpos.0, wpos.1, wpos.2), zone_entity_id);
    let _ = Prefabs::spawn_world(world, PrefabId::StairUp, config);
} else if room_coords.1 == max_y {
    let config = SpawnConfig::new((wpos.0, wpos.1, wpos.2), zone_entity_id);
    let _ = Prefabs::spawn_world(world, PrefabId::StairDown, config);
}
```

## Module Organization

The prefab system is organized into separate files in `src/domain/world/prefabs/`:

- `mod.rs` - Module exports and re-exports
- `prefabs.rs` - Core types (PrefabId, SpawnConfig, Prefabs resource)
- `spawn_prefab_cmd.rs` - Command implementation
- `pine_tree.rs` - Pine tree spawn function
- `boulder.rs` - Boulder spawn function  
- `bandit.rs` - Bandit spawn function
- `stair_down.rs` - Stairs down spawn function
- `stair_up.rs` - Stairs up spawn function

Each spawn function follows the signature:
```rust
pub fn spawn_[prefab](entity: Entity, world: &mut World, config: SpawnConfig)
```

## Key Features

### Type Safety
- Simple enum-based IDs prevent typos and invalid prefab references
- Rust compiler catches missing prefab types at compile time
- IDE auto-completion for all available prefabs

### Performance
- Zero runtime overhead for prefab lookup (HashMap with enum keys)
- Function pointers avoid dynamic dispatch
- No reflection or runtime type checking needed

### Command Pattern Benefits
- Deferred execution allows spawning during system execution
- Commands can be queued and executed when world access is available
- Clear separation between spawn request and execution

### Flexibility
- `SpawnConfig` allows extensive customization without changing core types
- Variants enable different versions of the same prefab type
- Metadata system supports arbitrary key-value configuration

### Maintainability
- Clear separation between prefab identification and spawning logic
- Easy to add new prefab types by extending enums and adding spawn functions
- Centralized registration makes system discoverable

### Integration
- Works seamlessly with Bevy ECS architecture
- Compatible with the serialization system (SaveFlag, RecordZonePosition)
- Fits well with zone-based world generation
- Supports both Commands and World-based spawning

## Usage Patterns

### Commands-based Spawning (Deferred)
```rust
// For use within systems that have access to Commands
let config = SpawnConfig::new((x, y, z), zone_entity);
let entity = prefabs.spawn(&mut commands, PrefabId::PineTree, config);
```

### World-based Spawning (Immediate)
```rust
// For use when you have direct World access (like in zone generation)
let config = SpawnConfig::new((x, y, z), zone_entity);
let entity = Prefabs::spawn_world(world, PrefabId::Boulder, config)?;
```

### Configuration Chaining
```rust
let config = SpawnConfig::new((x, y, z), zone_entity)
    .with_variant("large".to_string())
    .with_level(5)
    .with_custom_color(Palette::Red);
```

## Adding New Prefabs

1. **Add enum variant** to `PrefabId` in `prefabs.rs`
2. **Create spawn function file** (e.g., `spawn_new_entity.rs`)
3. **Implement spawn function** with signature `fn(Entity, &mut World, SpawnConfig)`
4. **Register prefab** in `register_all_prefabs()` method
5. **Export function** in `mod.rs`

## Current Implementation Status

The QuadBoy prefab system is fully implemented with the following prefabs:
- **PineTree** - Trees with green color and tree glyph
- **Boulder** - Rocks with brown color and rock glyph  
- **Bandit** - Enemies with red color and energy system
- **StairUp** - Upward stairs with brown color
- **StairDown** - Downward stairs with brown color

All zone generation has been migrated to use the prefab system instead of manual entity spawning.