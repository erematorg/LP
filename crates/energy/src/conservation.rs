use bevy::prelude::*;

/// Component tracking energy in a system
#[derive(Component, Debug, Clone, Copy)]
pub struct EnergyQuantity {
    /// Energy amount in joules
    pub value: f32,
}

/// Component representing energy transfer between entities
#[derive(Component, Debug)]
pub struct EnergyTransfer {
    /// Amount of energy transferred (joules)
    pub amount: f32,
    /// Source entity
    pub source: Entity,
    /// Target entity
    pub target: Entity,
}

/// Generic component for tracking energy transformations
/// Each specific energy module can define its own implementation
#[derive(Component, Debug)]
pub struct EnergyConversion {
    /// Efficiency of the conversion process (0.0-1.0)
    pub efficiency: f32,
    /// Energy lost during conversion (joules)
    pub losses: f32,
}

/// Verify energy conservation in a closed system
pub fn verify_conservation(
    initial_energy: f32,
    final_energy: f32,
    tolerance: f32,
) -> bool {
    // First law: Energy cannot be created or destroyed
    (final_energy - initial_energy).abs() <= tolerance
}

/// Calculate efficiency of an energy conversion process
pub fn conversion_efficiency(
    energy_input: f32,
    energy_output: f32,
) -> f32 {
    if energy_input > 0.0 {
        (energy_output / energy_input).clamp(0.0, 1.0)
    } else {
        0.0
    }
}