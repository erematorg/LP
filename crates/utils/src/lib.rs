pub mod cutoff;
pub mod pool;
pub mod spatial;
pub mod units;

use bevy::prelude::*;
use spatial::unified::{
    attach_spatial_cells, refresh_spatial_index_policy, remove_from_index_on_marker_removed,
    update_spatial_index,
};

/// Plugin for registering shared utility components
pub struct UtilsPlugin;

impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Spatial indexing
            .init_resource::<spatial::unified::UnifiedSpatialIndex>()
            .init_resource::<spatial::unified::NeighborSearchConfig>()
            .register_type::<spatial::unified::SpatiallyIndexed>()
            .register_type::<spatial::unified::SpatialCell>()
            .register_type::<spatial::unified::NeighborSearchMode>()
            .register_type::<spatial::unified::NeighborSearchConfig>()
            .register_type::<spatial::grid::GridCell>()
            .register_type::<pool::Pooled>()
            // Units and scale
            .init_resource::<units::PhysicsScale>()
            .register_type::<units::PhysicsScale>()
            // Configure system sets in PreUpdate
            .configure_sets(
                PreUpdate,
                (
                    spatial::unified::SpatialIndexSet::InjectMarkers,
                    spatial::unified::SpatialIndexSet::Maintain,
                )
                    .chain(),
            )
            // Index maintenance in PreUpdate (before physics reads it)
            .add_systems(
                PreUpdate,
                (
                    attach_spatial_cells,
                    update_spatial_index,
                    remove_from_index_on_marker_removed,
                    refresh_spatial_index_policy,
                )
                    .chain()
                    .in_set(spatial::unified::SpatialIndexSet::Maintain),
            );

        // Debug validation (only in debug builds)
        #[cfg(debug_assertions)]
        app.add_systems(Startup, units::validate_physics_scale);
    }
}

pub use cutoff::force_switch;
pub use pool::{EntityPool, Pooled};
pub use spatial::grid::{GridCell, SpatialGrid};
pub use spatial::unified::{
    NeighborSearchConfig, NeighborSearchMode, SpatialCell, SpatialIndexSet, SpatiallyIndexed,
    UnifiedSpatialIndex,
};
pub use units::{PhysicsScale, physics_to_render, render_to_physics};
