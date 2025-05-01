use bevy::prelude::*;

/// Component marking systems in thermal equilibrium
#[derive(Component, Debug)]
pub struct ThermalEquilibrium {
    pub connected_entities: Vec<Entity>,
}

/// Component for phase state of matter that will use the matter crate later once implemented
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseState {
    Solid,
    Liquid,
    Gas,
    Plasma,
}

/// Weighted equilibrium parameters
#[derive(Component, Debug, Clone, Copy)]  
pub struct ThermalProperties {
    pub thermal_mass: f32,
}

/// Check if two systems are in thermal equilibrium
pub fn is_in_equilibrium(
    temp_a: f32,
    temp_b: f32,
    props_a: &ThermalProperties,
    props_b: &ThermalProperties,
    tolerance: f32
) -> bool {
    // Weighted equilibrium considers both temperature and thermal properties
    let weighted_diff = (temp_a - temp_b).abs() /
        (1.0 + (props_a.thermal_mass * props_b.thermal_mass).sqrt());
    weighted_diff <= tolerance  
}

pub fn equilibrium_time_estimate(
    temp_diff: f32, // Initial temperature difference
    props_a: &ThermalProperties,
    props_b: &ThermalProperties,
    heat_transfer_rate: f32, // Rate of heat transfer (W)  
) -> f32 {
    // More sophisticated estimate considering thermal masses
    let combined_thermal_mass = props_a.thermal_mass + props_b.thermal_mass;
    if heat_transfer_rate > 0.0 { 
        // Weighted by combined thermal mass
        combined_thermal_mass * temp_diff / heat_transfer_rate
    } else {
        f32::INFINITY
    }
}