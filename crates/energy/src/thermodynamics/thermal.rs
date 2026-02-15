use bevy::prelude::*;
use matter::geometry::Radius;
use std::collections::HashMap;
use utils::{SpatialIndexSet, SpatiallyIndexed, UnifiedSpatialIndex, force_switch};

use crate::conservation::{EnergyQuantity, EnergyType};
use crate::pairwise::{
    PairwiseDeterminismConfig, for_each_neighbor_candidate, is_forward_entity_pair,
    prepare_sorted_entities_from_keys, prepare_staging_map,
};

/// Configuration for Fourier conduction system.
///
/// **LP-0 SCAFFOLDING**: Pairwise particle-particle approximation.
/// Future: Grid-based diffusion PDE (∇·(k∇T) = ρc_p ∂T/∂t).
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ThermalConductionConfig {
    /// Cutoff radius for thermal transfer (meters).
    /// **PERFORMANCE APPROXIMATION**: IRL heat conduction has no cutoff.
    /// **Units**: meters (m)
    pub cutoff_radius: f32,

    /// Force-switch start radius (meters).
    /// **PERFORMANCE APPROXIMATION**: C¹ smooth cutoff for numerical stability.
    /// **Units**: meters (m)
    pub switch_on_radius: f32,

    /// CFL safety factor (dimensionless). For dt ≤ safety × dx²/α.
    /// **Units**: dimensionless
    pub cfl_safety_factor: f32,

    /// Effective thermal conductivity for LP-0 pairwise conduction.
    /// **Units**: Watts / (meter × Kelvin) (W/(m·K))
    /// **Note**: Scaffold parameter. Future: per-material property in continuum/PDE.
    pub thermal_conductivity_w_per_m_k: f32,
}

impl Default for ThermalConductionConfig {
    fn default() -> Self {
        let cutoff = 10.0;
        Self {
            cutoff_radius: cutoff,
            switch_on_radius: 0.8 * cutoff,
            cfl_safety_factor: 0.5,
            thermal_conductivity_w_per_m_k: 1.0, // Explicit default, SI documented
        }
    }
}

/// Lightweight runtime sanity checks for thermal state.
///
/// These checks are O(N) and intended for realtime runs.
/// They are not full conservation diagnostics.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ThermalSanityConfig {
    /// Enable/disable sanity checks.
    pub enabled: bool,
    /// Warn if total thermal energy drops below this floor (J).
    pub min_total_thermal_energy_j: f32,
    /// Warn if total thermal energy grows more than this factor frame-to-frame.
    pub max_frame_growth_factor: f32,
    /// Warn if any temperature exceeds this bound (K).
    pub max_temperature_k: f32,
}

impl Default for ThermalSanityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_total_thermal_energy_j: 1e-3,
            max_frame_growth_factor: 2.0,
            max_temperature_k: 1e8,
        }
    }
}

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

#[derive(Default)]
pub(crate) struct ThermalComputeContext {
    thermal_data: HashMap<Entity, (Vec2, f32, f32, f32, f32)>,
    temp_changes: HashMap<Entity, f32>,
    sorted_entities: Vec<Entity>,
    neighbor_candidates: Vec<Entity>,
}

/// Mark thermal entities for spatial indexing.
///
/// **Phase A2**: Inject SpatiallyIndexed marker for UnifiedSpatialIndex.
/// This allows utils crate to manage spatial indexing without depending on Temperature.
fn mark_thermal_entities_spatially_indexed(
    mut commands: Commands,
    q: Query<Entity, (With<Temperature>, Without<SpatiallyIndexed>)>,
) {
    for e in q.iter() {
        commands.entity(e).insert(SpatiallyIndexed);
    }
}

/// Compute thermal transfer via Fourier's Law of conduction.
///
/// **LP-0 SCAFFOLDING**: Pairwise particle-particle thermal conduction.
/// **TEMPORARY**: Will be replaced by grid-based diffusion PDE (∇·(k∇T) = ρc_p ∂T/∂t) in LP-1.
///
/// **PHYSICS**: Fourier's Law q = k·A·ΔT/d (W), Q = P·dt (J)
/// - q: Heat flux (Watts) = k·A·ΔT/d
/// - Q: Energy transferred (Joules) = q·dt
/// - k: Thermal conductivity (W/(m·K))
/// - A: Contact area (m²) - assumes π·r² (spherical particles)
/// - ΔT: Temperature difference (K)
/// - d: Distance (m)
///
/// **APPROXIMATIONS**:
/// - Cutoff radius: 10m default (performance hack, IRL heat conduction has no cutoff)
/// - Contact area: A = π·min(r₁,r₂)² (assumes spherical particles)
/// - Constant c_p: Heat capacity assumed independent of temperature
///
/// **CONSERVATION**: Energy-symmetric (Q_out = Q_in), momentum conserved (no forces applied).
/// Not temperature-symmetric (hot bodies lose more than cold gain for same Q).
pub(crate) fn compute_fourier_conduction(
    mut commands: Commands,
    index: Res<UnifiedSpatialIndex>,
    config: Res<ThermalConductionConfig>,
    determinism: Res<PairwiseDeterminismConfig>,
    time: Res<Time>,
    mut thermal_transfer_events: MessageWriter<ThermalTransferEvent>,
    mut ctx: Local<ThermalComputeContext>,
    entities: Query<(
        Entity,
        &Transform,
        &Temperature,
        &ThermalConductivity,
        Option<&HeatCapacity>,
        Option<&Radius>,
    )>,
) {
    // **LP-0 SCAFFOLDING**: Pairwise particle-particle thermal conduction.
    // Future: Grid-based diffusion PDE (∇·(k∇T) = ρc_p ∂T/∂t).

    // Reuse staging buffers across frames to avoid per-frame allocation churn.
    let estimated = entities.iter().len();
    prepare_staging_map(&mut ctx.thermal_data, estimated);
    for (entity, trans, temp, conductivity, heat_capacity, radius) in entities.iter() {
        let pos = trans.translation.truncate();
        let k = conductivity.value;
        let t = temp.value;

        // No silent defaults: require HeatCapacity
        let Some(capacity) = heat_capacity else {
            #[cfg(debug_assertions)]
            panic!(
                "Entity {:?} missing HeatCapacity for thermal transfer",
                entity
            );

            #[cfg(not(debug_assertions))]
            {
                static LOGGED_CAPACITY: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if !LOGGED_CAPACITY.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    warn!("Skipping thermal entities missing HeatCapacity (logged once)");
                }
                continue;
            }
        };

        // No silent defaults: require Radius
        let Some(rad) = radius else {
            #[cfg(debug_assertions)]
            panic!(
                "Entity {:?} missing Radius for thermal contact area",
                entity
            );

            #[cfg(not(debug_assertions))]
            {
                static LOGGED_RADIUS: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                if !LOGGED_RADIUS.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    warn!("Skipping thermal entities missing Radius (logged once)");
                }
                continue;
            }
        };

        let c = capacity.value;
        let r = rad.value;
        ctx.thermal_data.insert(entity, (pos, t, k, c, r));
    }

    ctx.temp_changes.clear();
    let staged_count = ctx.thermal_data.len();
    ctx.temp_changes.reserve(staged_count);
    let dt = time.delta_secs();

    let cutoff_radius = config.cutoff_radius;
    let switch_on_radius = config.switch_on_radius;

    // Deterministic outer iteration: sort entities by stable id
    let staged_entities: Vec<Entity> = ctx.thermal_data.keys().copied().collect();
    prepare_sorted_entities_from_keys(&mut ctx.sorted_entities, staged_entities);

    // Iterate pairs via UnifiedSpatialIndex.
    let mut temp_changes = std::mem::take(&mut ctx.temp_changes);
    let thermal_data = std::mem::take(&mut ctx.thermal_data);
    let sorted_entities = std::mem::take(&mut ctx.sorted_entities);
    for &entity_a in &sorted_entities {
        let (pos_a, temp_a, k_a, capacity_a, radius_a) = thermal_data[&entity_a];
        // Find neighbors within cutoff using UnifiedSpatialIndex backend.
        for_each_neighbor_candidate(
            &index,
            pos_a,
            cutoff_radius,
            determinism.strict_neighbor_order,
            &mut ctx.neighbor_candidates,
            |entity_b| {
                // **Pair-once guarantee**: Only process pairs where B > A
                if !is_forward_entity_pair(entity_a, entity_b) {
                    return;
                }

                // Get data for entity B from staged map
                let Some((pos_b, temp_b, k_b, capacity_b, radius_b)) = thermal_data.get(&entity_b)
                else {
                    return;
                };

                let r_vec = *pos_b - pos_a;
                let distance = r_vec.length();

                // Property-based softening: use smaller radius as minimum distance
                let softening = radius_a.min(*radius_b);
                if distance < softening || distance >= cutoff_radius {
                    return;
                }

                let temp_diff = temp_a - temp_b; // +: A hotter, -: B hotter

                // Cross-sectional contact area: A = π·r² where r = min(r1, r2)
                //
                // **LP-0 APPROXIMATION**: Assumes spherical particles in contact.
                // Future: MPM will use material point volume overlap.
                let contact_radius = radius_a.min(*radius_b);
                let contact_area = std::f32::consts::PI * contact_radius.powi(2);

                // Average thermal conductivity (harmonic mean more accurate, but arithmetic for simplicity)
                let k_avg = (k_a + k_b) / 2.0;

                // Fourier's Law: power = k·A·ΔT/d (Watts)
                //
                // **IRL PHYSICS**: Heat flux q = -k·∇T (Fourier's Law)
                // For 1D: q = k·(T_hot - T_cold)/d
                let power = k_avg * contact_area * temp_diff / distance; // W

                if !power.is_finite() {
                    return; // Skip non-finite power
                }

                // Apply C¹ force-switch for smooth cutoff
                let switch = force_switch(distance, switch_on_radius, cutoff_radius);
                let power_switched = power * switch;

                // Energy transferred this frame: Q = P·dt (Joules)
                //
                // **CORRECT UNITS**: Power (W) × time (s) = Energy (J)
                let heat_energy = power_switched * dt; // J

                // First Law of Thermodynamics: ΔU = Q, ΔT = Q/C
                //
                // **Energy-symmetric** (not temperature-symmetric):
                // - Entity A loses Q: ΔU_a = -Q, ΔT_a = -Q/C_a
                // - Entity B gains Q: ΔU_b = +Q, ΔT_b = +Q/C_b
                //
                // Energy is conserved: Q_out = Q_in
                // Temperature changes differ if C_a ≠ C_b (correct physics!)
                let delta_t_a = -heat_energy / capacity_a; // Cooling if heat_energy > 0
                let delta_t_b = heat_energy / capacity_b; // Heating if heat_energy > 0

                if !delta_t_a.is_finite() || !delta_t_b.is_finite() {
                    return; // Skip non-finite temperature changes
                }

                *temp_changes.entry(entity_a).or_insert(0.0) += delta_t_a;
                *temp_changes.entry(entity_b).or_insert(0.0) += delta_t_b;

                thermal_transfer_events.write(ThermalTransferEvent {
                    source: entity_a,
                    target: entity_b,
                    heat_flow: power_switched.abs(),
                });
            },
        );
    }
    ctx.thermal_data = thermal_data;
    ctx.sorted_entities = sorted_entities;

    // Apply temperature changes
    for (&entity, &delta) in &temp_changes {
        if let Ok((_, _, temp, _, _, _)) = entities.get(entity) {
            let new_temp = (temp.value + delta).max(0.0); // Clamp to 0 K (Third Law approximation)
            commands
                .entity(entity)
                .insert(Temperature { value: new_temp });
        }
    }
    ctx.temp_changes = temp_changes;

    // **LP-0**: Thermal energy U = m·c_p·T not yet synced to EnergyQuantity.
    // TODO: Add sync system (Changed<Temperature> → EnergyQuantity).
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
/// System to check CFL stability condition for thermal diffusion
/// Runs once at startup to warn if timestep may cause instability
fn check_thermal_stability(
    index: Res<UnifiedSpatialIndex>,
    time: Res<Time>,
    diffusivities: Query<&ThermalDiffusivity>,
) {
    if diffusivities.is_empty() {
        return; // No thermal objects yet
    }

    let dt = time.delta_secs();
    if dt == 0.0 {
        return; // First frame, time not initialized yet
    }

    let dx = index.cell_size();
    let safety_factor = 0.5; // Conservative CFL constant for explicit diffusion

    for diffusivity in diffusivities.iter() {
        let alpha = diffusivity.value;
        if alpha <= 0.0 {
            continue;
        }

        // CFL condition: dt <= C * dx² / α
        let max_stable_dt = safety_factor * dx * dx / alpha;

        if dt > max_stable_dt {
            warn!(
                "Thermal diffusion may be UNSTABLE!\n\
                 Current timestep: dt = {:.6} s\n\
                 Stability limit:  dt <= {:.6} s\n\
                 Thermal diffusivity: α = {:.2e} m²/s\n\
                 Grid spacing: dx = {:.2} m\n\
                 Simulation may diverge or explode. Consider:\n\
                   - Smaller timestep (use fixed timestep plugin)\n\
                   - Coarser grid (increase SpatialGrid cell_size)\n\
                   - Implicit solver (future MPM work)",
                dt, max_stable_dt, alpha, dx
            );
        }
    }
}

/// Sync Temperature changes to EnergyQuantity for conservation tracking.
///
/// **LP-0**: Calculates thermal energy as U = m·c_p·T (assumes constant c_p).
/// Future: Use enthalpy for phase changes, proper thermodynamic potentials.
///
/// **Efficiency**: Uses Changed<Temperature> for O(N_changed) instead of O(N).
/// **Conservation**: Thermal energy tracked, but not yet integrated with ledger.
fn sync_thermal_energy(
    mut commands: Commands,
    changed_temps: Query<(Entity, &Temperature, &HeatCapacity), Changed<Temperature>>,
) {
    // **LP-0 SCAFFOLDING**: U = C·T (simplified thermal energy).
    // HeatCapacity already includes mass: C = m·c_p (J/K).
    // Future: Use proper thermodynamic potentials (enthalpy for constant P, etc.).

    for (entity, temp, heat_cap) in changed_temps.iter() {
        // Thermal energy: U = C·T (Joules)
        // where C = HeatCapacity = m·c_p (J/K), already accounts for mass.
        //
        // **APPROXIMATION**: Assumes constant c_p (valid for small ΔT).
        // Assumes T_ref = 0 K (absolute thermal energy).
        //
        // **IRL PHYSICS**: For phase changes, need enthalpy H = U + PV.
        // For proper thermodynamics, need U(S,V) or H(S,P).
        let thermal_energy = heat_cap.value * temp.value;

        commands.entity(entity).insert(EnergyQuantity {
            value: thermal_energy,
            energy_type: EnergyType::Thermal,
            max_capacity: None, // No max capacity for thermal energy
        });
    }
}

/// Realtime O(N) sanity checks for thermal state.
///
/// This catches obvious instability and data corruption early without
/// introducing expensive diagnostics.
fn check_thermal_sanity_realtime(
    config: Res<ThermalSanityConfig>,
    thermal_entities: Query<(Entity, &Temperature, Option<&HeatCapacity>)>,
    mut prev_total_energy: Local<Option<f32>>,
) {
    if !config.enabled {
        return;
    }

    let mut total_energy = 0.0f32;
    let mut hottest = 0.0f32;

    for (entity, temp, heat_capacity) in thermal_entities.iter() {
        if !temp.value.is_finite() || temp.value < 0.0 {
            warn!(
                "Thermal sanity: non-finite or negative temperature on entity {:?}: {} K",
                entity, temp.value
            );
            continue;
        }

        hottest = hottest.max(temp.value);

        if let Some(capacity) = heat_capacity {
            if capacity.value.is_finite() && capacity.value > 0.0 {
                total_energy += capacity.value * temp.value;
            }
        }
    }

    if !total_energy.is_finite() {
        warn!("Thermal sanity: total thermal energy is non-finite");
    } else if total_energy < config.min_total_thermal_energy_j {
        warn!(
            "Thermal sanity: total thermal energy too low: {:.6} J",
            total_energy
        );
    }

    if let Some(previous) = *prev_total_energy {
        if previous > f32::EPSILON {
            let growth = total_energy / previous;
            if growth.is_finite() && growth > config.max_frame_growth_factor {
                warn!(
                    "Thermal sanity: frame energy jump {:.3}x (prev {:.6} J -> now {:.6} J)",
                    growth, previous, total_energy
                );
            }
        }
    }
    *prev_total_energy = Some(total_energy);

    if hottest > config.max_temperature_k {
        warn!(
            "Thermal sanity: hottest temperature {:.3} K exceeds bound {:.3} K",
            hottest, config.max_temperature_k
        );
    }
}

pub struct ThermalSystemPlugin;

impl Plugin for ThermalSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThermalConductionConfig>()
            .init_resource::<ThermalSanityConfig>()
            .register_type::<ThermalConductionConfig>()
            .register_type::<ThermalSanityConfig>()
            .register_type::<Temperature>()
            .register_type::<ThermalConductivity>()
            .register_type::<ThermalDiffusivity>()
            .register_type::<Emissivity>()
            .register_type::<HeatCapacity>()
            .add_message::<ThermalTransferEvent>()
            .add_systems(Startup, check_thermal_stability)
            // Marker injection in PreUpdate
            .add_systems(
                PreUpdate,
                mark_thermal_entities_spatially_indexed.in_set(SpatialIndexSet::InjectMarkers),
            )
            // Thermal conduction → flush commands → sync energy.
            // conduction inserts Temperature via Commands; apply_deferred flushes
            // so sync_thermal_energy sees Changed<Temperature> in the same frame.
            .add_systems(
                Update,
                (
                    compute_fourier_conduction,
                    ApplyDeferred,
                    sync_thermal_energy,
                    check_thermal_sanity_realtime,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::thermal_utils::*;
    use super::*;

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

        assert!(
            (energy_lost - energy_gained).abs() < 1e-5,
            "Heat not conserved: lost {} != gained {}",
            energy_lost,
            energy_gained
        );
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

        assert!(
            (heat_flow - expected).abs() < 1e-5,
            "Fourier's law mismatch: {} != {}",
            heat_flow,
            expected
        );
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
        let expected = STEFAN_BOLTZMANN
            * emissivity
            * area
            * view_factor
            * (temp_emitter.powi(4) - temp_receiver.powi(4));

        assert!(
            (radiation - expected).abs() < 1e-3,
            "Stefan-Boltzmann mismatch"
        );
    }
}
