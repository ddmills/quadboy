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
- **States**: Game state management (main menu, play, explore, pause states)
- **UI**: Layout and user interface components

### Key Dependencies
- **macroquad**: Graphics framework and windowing
- **bevy_ecs**: Entity Component System
- **serde**: Serialization for save/load system
- Custom derive macro: `bevy_serializable_derive` for component serialization

### Entity Component System
The game uses Bevy ECS with a custom serialization system. Components must be registered with `SerializableComponentRegistry` for save/load functionality.

### Rendering System
- Text-based rendering using custom glyph system
- CRT shader for retro aesthetic
- Multiple render layers and targets
- Asset loading for tilesets and textures located in `src/assets/`

### Zone/World System
The game implements a zone-based world system where different areas can be loaded/unloaded dynamically. Events like `LoadZoneEvent` and `UnloadZoneEvent` manage this system.

### Build Notes
- Web builds require `wasm32-unknown-unknown` target and `wasm-bindgen`
- The build script handles WASM compilation and Macroquad-specific patches
- Clippy configuration allows higher complexity thresholds for game code