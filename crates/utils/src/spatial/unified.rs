//! Unified spatial indexing for physics participants only.
//!
//! - No crate cycles: this module knows nothing about Charge/Temperature/Mass
//! - Entities opt-in via `SpatiallyIndexed` marker (physics crates inject)
//! - Membership tracking: entity→cell map is authoritative

use bevy::prelude::*;
use std::collections::HashMap;

use super::grid::SpatialGrid;

/// System sets for spatial index maintenance.
///
/// **Execution order**: InjectMarkers → Maintain (in PreUpdate)
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SpatialIndexSet {
    /// Marker injection (physics crates insert SpatiallyIndexed)
    InjectMarkers,
    /// Index maintenance (attach/update/remove entities)
    Maintain,
}

/// Marker: entity participates in physics spatial queries.
///
/// Inserted by physics crates (energy/forces), never by utils.
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct SpatiallyIndexed;

/// Cached last-known cell for change detection.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct SpatialCell {
    pub cell: (i32, i32),
}

/// Unified spatial index with correct membership tracking.
///
/// **Correctness guarantees**:
/// - On insert/update: entity in exactly one cell
/// - On move: remove from old cell, insert into new cell
/// - No stale membership, no duplicates
#[derive(Resource)]
pub struct UnifiedSpatialIndex {
    grid: SpatialGrid,
    entity_cells: HashMap<Entity, (i32, i32)>,
}

impl UnifiedSpatialIndex {
    pub fn new(cell_size_meters: f32) -> Self {
        Self {
            grid: SpatialGrid::new(cell_size_meters),
            entity_cells: HashMap::new(),
        }
    }

    /// Insert entity and return computed cell.
    pub fn insert(&mut self, entity: Entity, position: Vec2) -> (i32, i32) {
        let cell = self.grid.world_to_grid(position);

        // Handle duplicate insertion (idempotent)
        if let Some(old) = self.entity_cells.insert(entity, cell) {
            if old != cell {
                self.grid.remove_from_cell(entity, old);
            }
        }

        self.grid.insert_in_cell(entity, cell);
        cell
    }

    /// Update entity position and return current cell.
    pub fn update(&mut self, entity: Entity, position: Vec2) -> (i32, i32) {
        let new_cell = self.grid.world_to_grid(position);

        match self.entity_cells.get(&entity).copied() {
            Some(old_cell) if old_cell == new_cell => new_cell,
            Some(old_cell) => {
                self.grid.move_entity(entity, old_cell, new_cell);
                self.entity_cells.insert(entity, new_cell);
                new_cell
            }
            None => {
                // First-time tracking (or missed attach)
                self.grid.insert_in_cell(entity, new_cell);
                self.entity_cells.insert(entity, new_cell);
                new_cell
            }
        }
    }

    pub fn remove(&mut self, entity: Entity) {
        if let Some(cell) = self.entity_cells.remove(&entity) {
            self.grid.remove_from_cell(entity, cell);
        }
    }

    /// Query entities in radius. Domain filtering via staged maps.
    pub fn query_radius(&self, position: Vec2, radius: f32) -> impl Iterator<Item = Entity> + '_ {
        self.grid.get_entities_in_radius(position, radius)
    }

    /// Get the cell size in meters.
    ///
    /// **Phase A2**: Exposed for CFL stability checking in thermal system.
    pub fn cell_size(&self) -> f32 {
        self.grid.cell_size
    }
}

impl Default for UnifiedSpatialIndex {
    fn default() -> Self {
        // Explicit default, performance parameter
        Self::new(50.0)
    }
}

/// Attach SpatialCell for entities marked SpatiallyIndexed.
pub fn attach_spatial_cells(
    mut commands: Commands,
    mut index: ResMut<UnifiedSpatialIndex>,
    q: Query<(Entity, &Transform), (With<SpatiallyIndexed>, Without<SpatialCell>)>,
) {
    for (e, t) in q.iter() {
        let pos = t.translation.truncate();
        let cell = index.insert(e, pos); // Returns cell
        commands.entity(e).insert(SpatialCell { cell });
    }
}

/// Update index when spatially indexed entities move.
pub fn update_spatial_index(
    mut index: ResMut<UnifiedSpatialIndex>,
    mut q: Query<
        (Entity, &Transform, &mut SpatialCell),
        (With<SpatiallyIndexed>, Changed<Transform>),
    >,
) {
    for (e, t, mut cell) in q.iter_mut() {
        let pos = t.translation.truncate();
        let new_cell = index.update(e, pos); // Returns cell
        cell.cell = new_cell;
    }
}

/// Remove from index when marker removed or entity despawns.
pub fn remove_from_index_on_marker_removed(
    mut index: ResMut<UnifiedSpatialIndex>,
    mut removed: RemovedComponents<SpatiallyIndexed>,
) {
    for e in removed.read() {
        index.remove(e);
    }
}
