pub mod pool;
pub mod spatial;

use bevy::prelude::*;

/// Plugin for registering shared utility components
pub struct UtilsPlugin;

impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<spatial::grid::GridCell>()
            .register_type::<pool::Pooled>();
    }
}

pub use pool::{EntityPool, Pooled};
pub use spatial::grid::{GridCell, SpatialGrid};
