use bevy::prelude::*;
use utils::{GridCell, SpatialGrid};

#[derive(Resource, Deref, DerefMut)]
struct ThermalGrid(SpatialGrid);

// Physical constants
pub const STEFAN_BOLTZMANN: f32 = 5.67e-8; // W/(m²·K⁴)

// STABILITY: Explicit thermal diffusion requires dt <= C·dx²/α for stability,
// where α = k/(ρ·cp) is thermal diffusivity, dx is grid spacing, C ≈ 0.5 safety factor.
// Current implementation uses Time.delta_secs() without enforcement.
// TODO: Add adaptive time-stepping or warn if dt exceeds stability limit.

/// Temperature component for thermal systems
///
/// Third Law of Thermodynamics: Absolute zero (0 K) cannot be reached in finite steps.
/// TODO: Current implementation clamps T >= 0 K but lacks proper quantum/medium physics
///       for modeling approach to absolute zero. Full Third Law behavior requires:
///       - Quantum mechanical effects (Bose-Einstein condensates, superfluidity)
///       - Medium/material properties at ultra-low temperatures
///       - Awaiting MPM (Material Point Method) implementation for proper material physics
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct Temperature {
    /// Temperature in Kelvin
    pub value: f32,
}

impl Temperature {
    pub fn new(kelvin: f32) -> Self {
        debug_assert!(
            kelvin >= 0.0,
            "Temperature below absolute zero violates thermodynamics"
        );
        debug_assert!(
            kelvin < 1e8,
            "Temperature exceeds realistic stellar core bounds (~1e8 K)"
        );
        Self {
            value: kelvin.max(0.0), // Non-physical clamp to prevent T<0 until a proper low-temp model exists
        }
    }

    pub fn from_celsius(celsius: f32) -> Self {
        Self::new(celsius + 273.15)
    }

    pub fn to_celsius(&self) -> f32 {
        self.value - 273.15
    }
}

/// Thermal conductivity property
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct ThermalConductivity {
    /// W/(m·K)
    pub value: f32,
}

/// Thermal diffusivity property
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct ThermalDiffusivity {
    /// m²/s
    pub value: f32,
}

impl ThermalDiffusivity {
    /// Calculate thermal diffusivity
    pub fn calculate(
        conductivity: f32,  // Thermal conductivity (W/(m·K))
        density: f32,       // Density (kg/m³)
        specific_heat: f32, // Specific heat capacity (J/(kg·K))
    ) -> Self {
        Self {
            value: conductivity / (density * specific_heat).max(f32::EPSILON),
        }
    }
}

/// Emissivity property for radiation calculations
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct Emissivity {
    /// Dimensionless value between 0.0 and 1.0
    /// 0.0 = perfect reflector, 1.0 = perfect emitter (black body)
    pub value: f32,
}

impl Emissivity {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }
}

/// Heat capacity - thermal inertia of an object
/// Determines how much energy is needed to change temperature
#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct HeatCapacity {
    /// J/K (Joules per Kelvin)
    /// For a material: C = m × c where m=mass (kg), c=specific heat (J/(kg·K))
    pub value: f32,
}

impl HeatCapacity {
    /// Create from mass and specific heat capacity
    /// Example: 1 kg of water with c=4184 J/(kg·K) → 4184 J/K
    pub fn from_material(mass: f32, specific_heat: f32) -> Self {
        Self {
            value: mass * specific_heat,
        }
    }

    /// Common materials (per kg)
    pub fn water(mass: f32) -> Self {
        Self::from_material(mass, 4184.0) // J/(kg·K)
    }

    pub fn air(mass: f32) -> Self {
        Self::from_material(mass, 1005.0) // J/(kg·K)
    }

    pub fn iron(mass: f32) -> Self {
        Self::from_material(mass, 449.0) // J/(kg·K)
    }

    pub fn aluminum(mass: f32) -> Self {
        Self::from_material(mass, 897.0) // J/(kg·K)
    }
}

/// Event for thermal energy transfer between entities
#[derive(Message, Debug)]
pub struct ThermalTransferEvent {
    /// Source entity losing thermal energy
    pub source: Entity,
    /// Target entity receiving thermal energy
    pub target: Entity,
    /// Amount of heat transferred
    pub heat_flow: f32,
}

use std::collections::HashMap;

fn update_thermal_grid(
    mut grid: ResMut<ThermalGrid>,
    mut query: Query<(Entity, &Transform, &mut GridCell), (With<Temperature>, Changed<Transform>)>,
) {
    for (entity, transform, mut cell) in query.iter_mut() {
        let position = transform.translation.truncate();
        let new_cell = grid.world_to_grid(position);
        if new_cell != cell.cell {
            grid.move_entity(entity, cell.cell, new_cell);
            cell.cell = new_cell;
        }
    }
}

fn attach_grid_cells_to_temperatures(
    mut commands: Commands,
    mut grid: ResMut<ThermalGrid>,
    query: Query<(Entity, &Transform), (With<Temperature>, Without<GridCell>)>,
) {
    for (entity, transform) in query.iter() {
        let position = transform.translation.truncate();
        let cell = grid.world_to_grid(position);
        grid.insert_in_cell(entity, cell);
        commands.entity(entity).insert(GridCell { cell });
    }
}

pub fn calculate_thermal_transfer(
    mut commands: Commands,
    grid: Res<ThermalGrid>,
    time: Res<Time>,
    mut thermal_transfer_events: MessageWriter<ThermalTransferEvent>,
    query: Query<(Entity, &Transform, &Temperature, &ThermalConductivity, Option<&HeatCapacity>)>,
) {
    let mut temp_changes: HashMap<Entity, f32> = HashMap::new();
    let mut processed_pairs = std::collections::HashSet::new();

    for (entity, transform, temp, conductivity, heat_capacity) in query.iter() {
        let position = transform.translation.truncate();
        let neighbors = grid.get_neighbors(position);

        for neighbor_entity in neighbors {
            if neighbor_entity == entity { continue; }
            if neighbor_entity.index() < entity.index() { continue; }

            let pair = (entity.index().min(neighbor_entity.index()), entity.index().max(neighbor_entity.index()));
            if !processed_pairs.insert(pair) { continue; }

            if let Ok((_, neighbor_transform, neighbor_temp, neighbor_conductivity, neighbor_heat_capacity)) = query.get(neighbor_entity) {
                let neighbor_pos = neighbor_transform.translation.truncate();
                let distance = position.distance(neighbor_pos);

                if distance < f32::EPSILON { continue; }

                let temp_diff = temp.value - neighbor_temp.value;
                let avg_conductivity = (conductivity.value + neighbor_conductivity.value) / 2.0;

                // Fourier's Law: q = k·A·ΔT/d
                // We assume normalized contact area A = 1 m² for simplicity
                // (actual area would depend on cell geometry in full 3D)
                let heat_flow = avg_conductivity * temp_diff / distance;

                if !heat_flow.is_finite() {
                    continue; // Skip non-finite heat flow
                }

                if heat_flow.abs() > f32::EPSILON {
                    // Energy transferred: Q = heat_flow × time (Joules)
                let heat_energy = heat_flow * time.delta_secs();
                // TODO: Thermal energy bookkeeping: U = m*cp*T not synced to EnergyQuantity/ledger; ΔT = Q/C uses fallback C if missing.

                    // First Law of Thermodynamics: ΔT = Q / C
                    // where C is heat capacity (J/K)
                    // If no HeatCapacity component: fallback to C = 1 J/K (abstract/reference object)
                    let capacity_a = heat_capacity.map(|c| c.value).unwrap_or(1.0);
                    let capacity_b = neighbor_heat_capacity.map(|c| c.value).unwrap_or(1.0);

                    let temp_change_a = heat_energy / capacity_a;
                    let temp_change_b = heat_energy / capacity_b;

                    if !temp_change_a.is_finite() || !temp_change_b.is_finite() {
                        continue; // Skip non-finite temperature changes
                    }

                    *temp_changes.entry(entity).or_insert(0.0) -= temp_change_a;
                    *temp_changes.entry(neighbor_entity).or_insert(0.0) += temp_change_b;

                    thermal_transfer_events.write(ThermalTransferEvent {
                        source: entity,
                        target: neighbor_entity,
                        heat_flow: heat_flow.abs(),
                    });
                }
            }
        }
    }

    for (entity, delta) in temp_changes {
        if let Ok((_, _, temp, _, _)) = query.get(entity) {
            commands.entity(entity).insert(Temperature { value: temp.value + delta });
        }
    }
}

/// Utility functions for thermal calculations
pub mod thermal_utils {
    use super::*;

    /// Calculate heat transfer via conduction
    pub fn heat_conduction(
        temp_diff: f32,    // Temperature difference (K)
        area: f32,         // Contact area (m²)
        distance: f32,     // Material thickness (m)
        conductivity: f32, // Thermal conductivity (W/(m·K))
    ) -> f32 {
        // q = k·A·ΔT/d (W)
        conductivity * area * temp_diff / distance.max(f32::EPSILON)
    }

    /// Calculate heat transfer via radiation
    pub fn heat_radiation(
        emitter_temp: f32,  // Temperature of emitting body (K)
        receiver_temp: f32, // Temperature of receiving body (K)
        area: f32,          // Surface area of emitting body (m²)
        emissivity: f32,    // Emissivity of emitting body (0.0-1.0)
        view_factor: f32,   // Geometric view factor (0.0-1.0)
    ) -> f32 {
        let t1_4 = emitter_temp.powi(4);
        let t2_4 = receiver_temp.powi(4);

        STEFAN_BOLTZMANN * emissivity * area * view_factor * (t1_4 - t2_4)
    }
}

/// Plugin for thermal system management
pub struct ThermalSystemPlugin;

impl Plugin for ThermalSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ThermalGrid(SpatialGrid::new(50.0)))
            .register_type::<Temperature>()
            .register_type::<ThermalConductivity>()
            .register_type::<ThermalDiffusivity>()
            .register_type::<Emissivity>()
            .register_type::<HeatCapacity>()
            .add_message::<ThermalTransferEvent>()
            .add_systems(Update, (
                attach_grid_cells_to_temperatures,
                update_thermal_grid,
                calculate_thermal_transfer,
            ).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::thermal_utils::*;

    #[test]
    fn test_heat_conservation() {
        // Test that heat lost by hot body = heat gained by cold body (within tolerance)
        let temp_hot = 400.0; // K
        let temp_cold = 300.0; // K
        let conductivity = 50.0; // W/(m·K)
        let distance = 1.0; // m
        let area = 1.0; // m²
        let dt = 0.1; // s

        // Fourier's law: q = k·A·ΔT/d
        let heat_flow_rate = heat_conduction(temp_hot - temp_cold, area, distance, conductivity);
        let heat_energy = heat_flow_rate * dt;

        // Apply to bodies with equal heat capacities
        let capacity = 1000.0; // J/K

        let temp_change_hot = -heat_energy / capacity;
        let temp_change_cold = heat_energy / capacity;

        let new_temp_hot = temp_hot + temp_change_hot;
        let new_temp_cold = temp_cold + temp_change_cold;

        // Verify heat is conserved: energy lost = energy gained
        let energy_lost = capacity * (temp_hot - new_temp_hot);
        let energy_gained = capacity * (new_temp_cold - temp_cold);

        assert!((energy_lost - energy_gained).abs() < 1e-5, "Heat not conserved: lost {} != gained {}", energy_lost, energy_gained);
    }

    #[test]
    fn test_fouriers_law() {
        // Test that heat_conduction matches analytical q = k·A·ΔT/d
        let temp_diff = 50.0; // K
        let area = 2.0; // m²
        let distance = 0.5; // m
        let conductivity = 100.0; // W/(m·K)

        let heat_flow = heat_conduction(temp_diff, area, distance, conductivity);

        // Expected: 100 * 2.0 * 50.0 / 0.5 = 20000 W
        let expected = conductivity * area * temp_diff / distance;

        assert!((heat_flow - expected).abs() < 1e-5, "Fourier's law mismatch: {} != {}", heat_flow, expected);
    }

    #[test]
    fn test_stefan_boltzmann_radiation() {
        // Test radiation heat transfer formula
        use super::thermal_utils::heat_radiation;

        let temp_emitter = 500.0; // K
        let temp_receiver = 300.0; // K
        let area = 1.0; // m²
        let emissivity = 0.9;
        let view_factor = 1.0;

        let radiation = heat_radiation(temp_emitter, temp_receiver, area, emissivity, view_factor);

        // Expected: σ·ε·A·F·(T₁⁴ - T₂⁴)
        let expected = STEFAN_BOLTZMANN * emissivity * area * view_factor * (temp_emitter.powi(4) - temp_receiver.powi(4));

        assert!((radiation - expected).abs() < 1e-3, "Stefan-Boltzmann mismatch");
    }
}
