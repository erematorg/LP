pub mod fields;
pub mod interactions;

use bevy::prelude::*;

// NOTE: Charge is NOT conserved; EM is quasi-static (no charge continuity equation).
// Ref: .claude/skills/lp-physics-chem-invariants/references/em.md
pub struct ElectromagnetismPlugin;

impl Plugin for ElectromagnetismPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<fields::ElectricField>()
            .register_type::<fields::MagneticField>()
            .register_type::<interactions::ElectromagneticWave>()
            .register_type::<interactions::MaterialProperties>()
            .add_message::<fields::ElectromagneticFieldInteractionEvent>()
            .add_systems(Update, fields::calculate_field_interactions);
    }
}

/// The electromagnetism prelude.
///
/// This includes the most common types for electromagnetic systems.
pub mod prelude {
    pub use crate::electromagnetism::fields::{ElectricField, MagneticField};
    pub use crate::electromagnetism::interactions::{ElectromagneticWave, MaterialProperties};
}
