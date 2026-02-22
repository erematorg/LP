# Utils

Spatial indexing and object pooling for ECS-based simulations.

## Core Modules

- `spatial::UnifiedSpatialIndex` - Hybrid spatial index (grid + optional tree backend)
- `pool::EntityPool` - Entity recycling for spawn/despawn

## Quick Start

```rust
app.add_plugins(utils::UtilsPlugin);
```

## Scope & Limits

- UnifiedSpatialIndex: 2D, point-entity based
- Radius queries return candidates; exact distance filtering is caller's responsibility
- Tuned for ECS at LP-0 scale; high-N (10k+) systems will use SoA/MPM directly
- Large bodies or 3D need custom structures

## Design

**UnifiedSpatialIndex backends:**
- UniformCellField: dense, dynamic
- HierarchicalEnvelopeField: sparse, non-uniform
- Adaptive: switches by observed density

**EntityPool:** Hard pool (strips components); for spawn rates <100/frame

## Future

- for_each_in_aabb query path (MPM coupling)
- Strict-determinism mode (sorted neighbors by entity ID)
- 3D index primitives only when MPM moves to 3D
