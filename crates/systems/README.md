# Systems

High-level simulation orchestration: AI, acoustics, MPM, persistence.

## Core Modules

- `ai` - utility-driven agency for creatures/actors
- `acoustics` - physics-based sound (early-stage)
- `mpm` - Material Point Method solver (planned)
- `save_system` - save/load infrastructure

## Quick Start

```rust
use bevy::prelude::*;
use systems::SystemsPlugin;

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(SystemsPlugin::default())
    .run();
```

Each module can be enabled/disabled via builder methods.

## Scope & Limits

- Sits on top of domain crates (energy, forces, information, matter)
- Handles scheduling and plugin wiring only
- AI: utility-driven only (reactive agents, not planners)
- Acoustics: hooks to wave simulation when available
- MPM: MLS-MPM first; PB-MPM gated

## Status

Production-ready for: AI, save_system
Early-stage: acoustics, mpm
