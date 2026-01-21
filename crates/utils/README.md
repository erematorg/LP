# Utils

Shared utilities and optimizations.

## Modules

- `spatial::SpatialGrid` - Sparse spatial hash grid for neighbor queries
- `pool::EntityPool` - Entity recycling for spawn/despawn scenarios

## Scope & Limits
- SpatialGrid and UnifiedSpatialIndex are 2D, point-entity hash grids; large bodies or 3D need other structures.
- Radius queries return candidate sets from grid cells; exact distance filtering remains a caller responsibility.
- ECS-based helpers are tuned for LP-0 scale; high-N systems will move to SoA/MPM paths.

## Usage

```rust
app.add_plugins(utils::UtilsPlugin);
```

## Scaling Patterns

### Current Implementation

**SpatialGrid:** Incremental updates for ECS queries (thermal, waves, AI)
- Handles 100-10k entities efficiently
- Prefer stable cell sizes to avoid churn; reuse a single grid per query domain to reduce allocations.

**EntityPool (Hard Pool):** Strips components on release
- Use for moderate spawn rates (<100/frame)
- Examples: thermal entities, wave centers, creature shells
- Prewarm pools to avoid runtime allocations; excess releases beyond capacity are despawned.

### Future Patterns

**Soft Pool** (100-1000/frame): Toggle `Inactive` marker, keep components
```rust
Query<&Transform, Without<Inactive>>
```

**SoA Resource** (10k+/frame): Bypass ECS for MPM core
```rust
struct ParticleSystem {
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
}
```

## Architecture

- **Mind (ECS):** Behavior, AI -> SpatialGrid + EntityPool
- **Body (SoA):** Physics, chemistry -> custom structures
