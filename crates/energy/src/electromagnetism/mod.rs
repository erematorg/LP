pub mod charges;
pub mod fields;
pub mod interactions;

use bevy::prelude::*;
use forces::PhysicsSet;
use forces::core::gravity::GravitySet;
use utils::SpatialIndexSet;

// NOTE: Charge is NOT conserved; EM is quasi-static (no charge continuity equation).
// Ref: .claude/skills/lp-physics-chem-invariants/references/em.md

// **LP-0 APPROXIMATION**: Quasi-static EM (v << c, no wave propagation).
// Future: Grid-based Poisson solve + wave equation if needed.
pub struct ElectromagnetismPlugin;

impl Plugin for ElectromagnetismPlugin {
    fn build(&self, app: &mut App) {
        app
            // LP-0: Coulomb forces between point charges (using UnifiedSpatialIndex)
            .init_resource::<charges::CoulombConfig>()
            .register_type::<charges::Charge>()
            .register_type::<charges::SofteningLength>()
            // Marker injection in PreUpdate
            .add_systems(
                PreUpdate,
                charges::mark_charged_entities_spatially_indexed
                    .in_set(SpatialIndexSet::InjectMarkers),
            )
            // Coulomb forces in Update, in force accumulation, after gravity
            .add_systems(
                Update,
                charges::apply_coulomb_pairwise_forces
                    .in_set(PhysicsSet::AccumulateForces)
                    .after(GravitySet::NBodyGravity),
            )
            // Keep field components for visualization
            .register_type::<fields::ElectricField>()
            .register_type::<fields::MagneticField>()
            .register_type::<interactions::ElectromagneticWave>()
            .register_type::<interactions::MaterialProperties>()
            .add_message::<fields::ElectromagneticFieldInteractionEvent>();

        // **DISABLED**: Field-field interactions violate Maxwell's equations.
        // Fields don't interact with each other (they superpose linearly).
        // .add_systems(Update, fields::calculate_field_interactions);
    }
}

/// The electromagnetism prelude.
///
/// This includes the most common types for electromagnetic systems.
pub mod prelude {
    pub use crate::electromagnetism::charges::{Charge, CoulombConfig, SofteningLength};
    pub use crate::electromagnetism::fields::{ElectricField, MagneticField};
    pub use crate::electromagnetism::interactions::{ElectromagneticWave, MaterialProperties};
}
