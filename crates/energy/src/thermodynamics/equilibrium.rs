use bevy::prelude::*;

/// Component marking systems in thermal equilibrium
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ThermalEquilibrium {
    pub connected_entities: Vec<Entity>,
    /// Equilibrium group ID for transitivity tracking
    pub group_id: Option<u32>,
}

/// Component for phase state of matter that will use the matter crate later once implemented, soon once PBMPM will be in place
/// and the matter crate is implemented
/// Stub phase state representation until matter integration is ready.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
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
/// Zeroth Law: Equilibrium means equal temperature (within tolerance)
pub fn is_in_equilibrium(
    temp_a: f32,
    temp_b: f32,
    _props_a: &ThermalProperties,
    _props_b: &ThermalProperties,
    tolerance: f32,
) -> bool {
    // Zeroth Law: Thermal equilibrium is defined purely by temperature equality
    // Thermal mass affects TIME to reach equilibrium, not the definition of equilibrium itself
    (temp_a - temp_b).abs() <= tolerance
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

/// Zeroth Law: Apply transitivity to thermal equilibrium relationships
/// If A is in equilibrium with C and B is in equilibrium with C, then A is in equilibrium with B
pub fn apply_equilibrium_transitivity(equilibrium_relationships: &mut Vec<(Entity, Entity)>) {
    let mut changed = true;

    // Keep applying transitivity until no new relationships are found
    while changed {
        changed = false;
        let current_relationships = equilibrium_relationships.clone();

        // For each pair of existing relationships
        for (a, c) in &current_relationships {
            for (b, c2) in &current_relationships {
                // If both A and B are in equilibrium with the same entity C
                if c == c2 && a != b {
                    let new_relationship = if a < b { (*a, *b) } else { (*b, *a) };

                    // Add the transitive relationship A↔B if it doesn't exist
                    if !equilibrium_relationships.contains(&new_relationship) {
                        equilibrium_relationships.push(new_relationship);
                        changed = true;
                    }
                }
            }
        }
    }
}

/// Find all entities in the same equilibrium group using transitivity
pub fn find_equilibrium_group(
    entity: Entity,
    equilibrium_relationships: &[(Entity, Entity)],
) -> Vec<Entity> {
    let mut group = vec![entity];
    let mut to_process = vec![entity];

    while let Some(current) = to_process.pop() {
        // Find all entities connected to current entity
        for (a, b) in equilibrium_relationships {
            let connected = if *a == current {
                Some(*b)
            } else if *b == current {
                Some(*a)
            } else {
                None
            };

            if let Some(connected_entity) = connected {
                if !group.contains(&connected_entity) {
                    group.push(connected_entity);
                    to_process.push(connected_entity);
                }
            }
        }
    }

    group
}

/// Validate that all entities in an equilibrium group have consistent temperatures
/// Core Zeroth Law requirement: transitivity must be physically valid
pub fn validate_equilibrium_group_consistency(
    entities: &[Entity],
    temperatures: &[(Entity, f32)],
    tolerance: f32,
) -> bool {
    // Zeroth Law: all entities in same group must have same temperature
    if let Some(first_temp) = temperatures.iter().find(|(e, _)| entities.contains(e)) {
        temperatures
            .iter()
            .filter(|(e, _)| entities.contains(e))
            .all(|(_, temp)| (temp - first_temp.1).abs() <= tolerance)
    } else {
        true
    }
}
