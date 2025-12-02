# Utils

Shared utilities and optimizations.

## Modules

- `spatial::SpatialGrid` - Sparse spatial hash grid for neighbor queries
- `pool::EntityPool` - Entity recycling for spawn/despawn scenarios

## Usage

```rust
app.add_plugins(utils::UtilsPlugin);
```

## Scaling Patterns

### Current Implementation

**SpatialGrid:** Incremental updates for ECS queries (thermal, waves, AI)
- Handles 100-10k entities efficiently

**EntityPool (Hard Pool):** Strips components on release
- Use for moderate spawn rates (<100/frame)
- Examples: thermal entities, wave centers, creature shells

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

- **Mind (ECS):** Behavior, AI → SpatialGrid + EntityPool
- **Body (SoA):** Physics, chemistry → custom structures
