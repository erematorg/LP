use bevy::prelude::*;
use crate::thermal::Temperature;

/// Entropy component for thermodynamic systems
#[derive(Component, Debug, Clone, Copy)]
pub struct Entropy {
    /// Entropy in J/K
    pub value: f32,
}

impl Entropy {
    pub fn new(value: f32) -> Self {
        Self { value: value.max(0.0) }
    }
}

/// Process reversibility characteristic
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reversibility {
    Reversible,
    Irreversible,
}

/// Calculate entropy change for heat transfer
pub fn entropy_change_heat_transfer(
    heat_transferred: f32,
    temperature: f32,
) -> f32 {
    // ΔS = Q/T
    if temperature > 0.0 {
        heat_transferred / temperature
    } else {
        0.0
    }
}

/// Check if entropy change violates the second law
pub fn is_valid_process(total_entropy_change: f32) -> bool {
    // Second law requires total entropy to increase or remain unchanged
    total_entropy_change >= 0.0
}