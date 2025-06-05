use bevy::prelude::*;

/// Acoustics plugin for physics-based sound generation
/// 
/// Note: Acoustics in LP are generated from fundamental physics:
/// - Sound waves are mechanical energy (from energy crate)  
/// - Propagation requires matter medium (from matter crate)
/// - All audio emerges from white noise + frequency filtering
/// - No hardcoded audio files - everything is procedurally generated
pub struct AcousticsPlugin;

impl Plugin for AcousticsPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Will integrate with energy crate's wave systems
        // TODO: Will require matter crate's medium properties for propagation
        // TODO: White noise generation + frequency filtering system
        app.register_type::<AcousticMedium>();
    }
}

/// Properties of matter that affect sound propagation
/// Will integrate with matter crate when ready
#[derive(Component, Debug, Clone, Reflect)]
pub struct AcousticMedium {
    /// Speed of sound in this medium (m/s)
    pub sound_speed: f32,
    /// Density affects impedance and reflection
    pub density: f32,
    /// How much energy is absorbed per distance
    pub absorption_coefficient: f32,
}

impl Default for AcousticMedium {
    fn default() -> Self {
        Self {
            sound_speed: 343.0, // Air at 20°C
            density: 1.225,     // Air density kg/m³
            absorption_coefficient: 0.01,
        }
    }
}

/// Prelude for acoustics (minimal for now)
pub mod prelude {
    pub use super::{
        AcousticsPlugin,
        AcousticMedium,
    };
}

// TODO: Future implementation will include:
// - Integration with energy::waves for wave propagation
// - Matter medium interaction for realistic sound physics  
// - White noise -> frequency filtering for emergent audio
// - Doppler effects, reflection, interference patterns
// - No audio files - pure procedural generation from physics