# Utils

Shared utilities and optimizations.

## Modules

- `spatial::UnifiedSpatialIndex` - Hybrid spatial index (grid hash + optional tree backend)
- `spatial::SpatialGrid` - Sparse spatial hash grid used by UnifiedSpatialIndex
- `pool::EntityPool` - Entity recycling for spawn/despawn scenarios

## Scope & Limits
- UnifiedSpatialIndex is 2D and point-entity based; large bodies or 3D need other structures.
- Radius queries return candidate sets from the active backend; exact distance filtering remains a caller responsibility.
- ECS-based helpers are tuned for LP-0 scale; high-N systems will move to SoA/MPM paths.
- LP-0 uses 2D neighbor search. If LP moves core continuum simulation to 3D MPM, this layer needs a 3D upgrade path.

## Usage

```rust
app.add_plugins(utils::UtilsPlugin);
```

## Scaling Patterns

### Current Implementation

**UnifiedSpatialIndex:** Runtime backend policy for ECS queries (thermal, EM, AI)
- `UniformCellField`: fastest for dense, highly dynamic domains.
- `HierarchicalEnvelopeField`: useful for sparse/non-uniform distributions.
- `Adaptive`: switches backend from observed density with cooldown to avoid thrashing.

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

### LP-0 Spatial TODO (Non-Overkill)

These are the next practical upgrades from the current hybrid index work:

- Add `for_each_in_aabb` query path for batch neighborhood pulls (important for later MPM coupling).
- Add an optional strict-determinism query mode that emits neighbors sorted by entity id (debug/replay builds).
- Tune tree split/rebuild heuristics using LP scene profiles before adding new backend complexity.
- Keep scope 2D for LP-0; only move to 3D index primitives when MPM integration starts.

## Architecture

- **Mind (ECS):** Behavior, AI -> UnifiedSpatialIndex + EntityPool
- **Body (SoA):** Physics, chemistry -> custom structures
