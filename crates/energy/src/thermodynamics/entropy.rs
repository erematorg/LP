use bevy::prelude::*;

/// Entropy component for thermodynamic systems
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct Entropy {
    /// Entropy in J/K
    pub value: f32,
}

impl Entropy {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.max(0.0),
        }
    }
}

/// Process reversibility characteristic
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Reversibility {
    Reversible,
    Irreversible,
}

/// Calculate entropy change for heat transfer
pub fn entropy_change_heat_transfer(heat_transferred: f32, temperature: f32) -> f32 {
    // ΔS = Q/T
    if temperature > 0.0 {
        heat_transferred / temperature
    } else {
        0.0
    }
}

/// Calculate entropy change for irreversible processes
pub fn entropy_change_irreversible(
    energy_transferred: f32,    // Energy transferred in process (J)
    source_temperature: f32,    // Temperature of source (K)
    sink_temperature: f32,      // Temperature of sink (K)
) -> f32 {
    // For irreversible processes, total entropy change is positive
    // ΔS = Q/Tcold - Q/Thot
    if source_temperature > 0.0 && sink_temperature > 0.0 {
        energy_transferred * (1.0/sink_temperature - 1.0/source_temperature)
    } else {
        0.0
    }
}

/// Check if entropy change violates the second law
pub fn is_valid_process(total_entropy_change: f32) -> bool {
    // Second law requires total entropy to increase or remain unchanged
    total_entropy_change >= 0.0
}

/// Calculate total entropy change of a system and its surroundings
pub fn total_entropy_change(
    system_entropy_change: f32,
    surroundings_entropy_change: f32,
) -> f32 {
    system_entropy_change + surroundings_entropy_change
}