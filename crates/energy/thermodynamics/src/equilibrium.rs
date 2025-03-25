use bevy::prelude::*;
use crate::thermal::Temperature;

/// Component marking systems in thermal equilibrium
#[derive(Component, Debug)]
pub struct ThermalEquilibrium {
    /// Other entities this system is in equilibrium with
    pub connected_entities: Vec<Entity>,
}

/// Component for phase state of matter
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseState {
    Solid,
    Liquid,
    Gas,
    Plasma,
}

/// Check if two systems are in thermal equilibrium
pub fn is_in_equilibrium(temp_a: f32, temp_b: f32, tolerance: f32) -> bool {
    // Zeroth law: Two systems are in thermal equilibrium if their temperatures are equal
    (temp_a - temp_b).abs() <= tolerance
}

/// Calculate time to reach thermal equilibrium
pub fn equilibrium_time_estimate(
    temp_diff: f32,      // Initial temperature difference
    thermal_mass: f32,   // Thermal mass (J/K)
    heat_transfer_rate: f32, // Rate of heat transfer (W)
) -> f32 {
    // Simple estimate based on temperature difference and heat transfer rate
    if heat_transfer_rate > 0.0 {
        thermal_mass * temp_diff / heat_transfer_rate
    } else {
        f32::INFINITY
    }
}